use crate::method::{Method, StateGetMetadata};
use alloc::sync::Arc;
use core::any::Any;
use core::fmt::Debug;
use futures::sync::oneshot::{self, channel, Receiver, Sender};
use futures::Future;
use node_template_runtime::GenesisConfig;
use std::net::{Ipv6Addr, SocketAddr, TcpListener};
use std::path::PathBuf;
use std::thread::spawn;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use substrate_network::construct_simple_protocol;
use substrate_service::{AbstractService, ChainSpec};
use tempdir::TempDir;

extern crate alloc;

pub struct RunningFullNode {
    addr: SocketAddr,
    #[allow(dead_code)]
    chain_store: TempDir,
    stop_tx: Option<Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl RunningFullNode {
    /// # Panics
    ///
    /// Panics if node fails to respond with metadata after startup.
    pub fn new() -> Self {
        let addr = any_available_local_addr();
        let chain_store = TempDir::new("").unwrap();
        let (stop_tx, stop_rx) = channel();
        let thread = spawn({
            let chain_store = chain_store.path().join("chain");
            let addr = addr.clone();
            move || log_err(run_node(addr, chain_store, stop_rx))
        });
        let ret = Self {
            addr,
            chain_store,
            stop_tx: Some(stop_tx),
            thread: Some(thread),
        };

        // wait for full-node to be running
        timeout(Duration::from_secs(10), || {
            ret.remote_call::<StateGetMetadata>([]).ok()
        })
        .expect("timeout while waiting for full node to start");

        ret
    }

    pub fn remote_call<M: Method>(&self, arg: M::Args) -> Result<M::Return, String> {
        crate::remote_call::call::<M>(&format!("ws://{}", self.addr), arg)
    }
}

impl Drop for RunningFullNode {
    fn drop(&mut self) {
        if self.remote_call::<StateGetMetadata>([]).is_err() {
            eprintln!("Full node was not functioning at the end of a test.");
            if !std::thread::panicking() {
                panic!()
            }
        }
        log_err(self.stop_tx.take().ok_or(()).and_then(|t| t.send(())));
        log_err(self.thread.take().ok_or(()).map(|t| log_err(t.join())));
    }
}

struct NoNativeExecutor;

impl substrate_executor::NativeExecutionDispatch for NoNativeExecutor {
    fn dispatch(
        _ext: &mut dyn substrate_executor::Externalities<substrate_executor::Blake2Hasher>,
        _method: &str,
        _data: &[u8],
    ) -> Result<Vec<u8>, substrate_executor::error::Error> {
        Err(substrate_executor::error::Error::Other(
            "this runtime is wasm only, but native executor was called",
        ))
    }

    fn native_version() -> substrate_executor::NativeVersion {
        node_template_runtime::native_version()
    }
}

// run node until 'close' receives an item or is cancelled or and error stops the node
fn run_node(
    addr: SocketAddr,
    chain_store: PathBuf,
    close: Receiver<()>,
) -> Result<(), &'static str> {
    let mut config = substrate_service::Configuration::<(), _>::default_with_spec(make_chainspec());
    config.rpc_ws = Some(addr);
    config.rpc_ws_max_connections = Some(1);
    config.rpc_cors = None; // all connections are allowed
    config.database_path = chain_store;

    let service = full_start(config)
        .map_err(|_| "failed to create service builder")?
        .map_err(|_| "error while running service");

    let stop = close.or_else(|oneshot::Canceled| Ok(()));

    stop.select(service)
        .wait()
        .map(|_| ())
        .map_err(|(err, _)| err)
}

fn make_chainspec() -> ChainSpec<GenesisConfig> {
    substrate_warmup_chaingen::chain_spec::Chain::Ved
        .generate()
        .into()
}

type OpaqueBlock = sr_primitives::generic::Block<
    sr_primitives::generic::Header<u32, sr_primitives::traits::BlakeTwo256>,
    sr_primitives::OpaqueExtrinsic,
>;

construct_simple_protocol! {
    pub struct NodeProtocol where Block = OpaqueBlock { }
}

fn full_start(
    config: substrate_service::config::Configuration<(), node_template_runtime::GenesisConfig>,
) -> Result<impl AbstractService, substrate_service::error::Error> {
    substrate_service::ServiceBuilder::new_full::<
        node_template_runtime::opaque::Block,
        node_template_runtime::RuntimeApi,
        NoNativeExecutor,
    >(config)?
    .with_select_chain(|_config, backend| Ok(substrate_client::LongestChain::new(backend.clone())))?
    .with_transaction_pool(|config, client| {
        Ok(substrate_transaction_pool::txpool::Pool::new(
            config,
            substrate_transaction_pool::ChainApi::new(client),
        ))
    })?
    .with_import_queue(|_config, client, mut select_chain, transaction_pool| {
        let select_chain = select_chain
            .take()
            .ok_or_else(|| substrate_service::Error::SelectChainRequired)?;
        let (block_import, _link_half) = substrate_finality_grandpa::block_import::<
            _,
            _,
            _,
            node_template_runtime::RuntimeApi,
            _,
            _,
        >(client.clone(), client.clone(), select_chain)?;
        let justification_import = block_import.clone();

        let (import_queue, _babe_link, _babe_block_import, _pruning_task) =
            substrate_consensus_babe::import_queue(
                substrate_consensus_babe::Config::get_or_compute(&*client)?,
                block_import,
                Some(Box::new(justification_import)),
                None,
                client.clone(),
                client,
                substrate_inherents::InherentDataProviders::new(),
                Some(transaction_pool),
            )?;

        Ok(import_queue)
    })?
    .with_network_protocol(|_| Ok(NodeProtocol::new()))?
    .with_finality_proof_provider(|client, backend| {
        Ok(Arc::new(
            substrate_finality_grandpa::FinalityProofProvider::new(backend, client),
        ))
    })?
    .build()
}

fn log(a: impl Debug) {
    eprintln!("{:?}", a)
}

fn log_err(a: Result<impl Any, impl Debug>) {
    let _ = a.map_err(log);
}

/// return some usable address on which to listen over tcp
fn any_available_local_addr() -> SocketAddr {
    let ret = {
        TcpListener::bind((Ipv6Addr::LOCALHOST, 0))
            .unwrap()
            .local_addr()
            .unwrap()
    };

    // test to make sure it works
    TcpListener::bind(ret).unwrap();

    ret
}

// Keep attempting f until f returns Ok or until timeout is reached
// returns Err on timeout, Ok on success
fn timeout<T>(timeout: Duration, f: impl Fn() -> Option<T>) -> Result<T, Duration> {
    let start = Instant::now();
    let end = start + timeout;
    while end > Instant::now() {
        match f() {
            Some(t) => return Ok(t),
            _ => {}
        };
    }
    Err(Instant::now() - start)
}

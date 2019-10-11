use alloc::sync::Arc;
use core::any::Any;
use core::fmt::Debug;
use futures::future::{err, ok};
use futures::sync::oneshot::{self, channel, Receiver, Sender};
use futures::Future;
use jsonrpc_client_transports::transports;
use jsonrpc_client_transports::RpcChannel;
use jsonrpc_client_transports::RpcError;
use node_template_runtime::GenesisConfig;
use sr_primitives::generic;
use std::net::{Ipv6Addr, SocketAddr, TcpListener};
use std::path::PathBuf;
use std::thread::spawn;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use substrate_client::LongestChain;
use substrate_executor::NativeVersion;
use substrate_executor::{Blake2Hasher, Externalities, NativeExecutionDispatch};
use substrate_finality_grandpa::FinalityProofProvider;
use substrate_inherents::InherentDataProviders;
use substrate_network::construct_simple_protocol;
use substrate_service::{AbstractService, ChainSpec};
use substrate_transaction_pool::{txpool::Pool, ChainApi};
use tempdir::TempDir;

extern crate alloc;

pub struct RunningFullNode {
    addr: SocketAddr,
    #[allow(dead_code)]
    store: TempDir, // temorary directory for chain db and keystore
    stop_tx: Option<Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl RunningFullNode {
    /// # Panics
    ///
    /// Panics if node fails to respond with metadata after startup.
    pub fn new() -> impl Future<Item = Self, Error = String> {
        let addr = any_available_local_addr();
        let store = TempDir::new("").unwrap();
        let (stop_tx, stop_rx) = channel();
        let thread = spawn({
            let store = store.path().to_path_buf().clone();
            let addr = addr.clone();
            move || log_err(run_node(addr, store, stop_rx))
        });
        let url = format!("ws://{}", addr).parse().unwrap();
        let ret = Self {
            addr,
            store,
            stop_tx: Some(stop_tx),
            thread: Some(thread),
        };

        // wait for full-node to be running
        timeout(Duration::from_secs(10), move || {
            transports::ws::connect::<RpcChannel>(&url)
                .map(|_| ())
                .map_err(|_| ())
        })
        .map_err(|duration| {
            format!(
                "timeout while waiting for RunningFullNode to start. Took {:?}",
                duration
            )
        })
        .map(|()| ret)
    }

    pub fn client_channel(&self) -> impl Future<Item = RpcChannel, Error = RpcError> {
        let url = format!("ws://{}", self.addr).parse().unwrap();
        transports::ws::connect(&url)
    }
}

impl Drop for RunningFullNode {
    fn drop(&mut self) {
        log_err(self.stop_tx.take().ok_or(()).and_then(|t| t.send(())));
        log_err(self.thread.take().ok_or(()).map(|t| log_err(t.join())));
    }
}

struct NoNativeExecutor;

impl NativeExecutionDispatch for NoNativeExecutor {
    fn dispatch(
        _ext: &mut dyn Externalities<Blake2Hasher>,
        _method: &str,
        _data: &[u8],
    ) -> Result<Vec<u8>, substrate_executor::error::Error> {
        Err(substrate_executor::error::Error::Other(
            "this runtime is wasm only, but native executor was called",
        ))
    }

    fn native_version() -> NativeVersion {
        node_template_runtime::native_version()
    }
}

// run node until 'close' receives an item or is cancelled or and error stops the node
fn run_node(addr: SocketAddr, store: PathBuf, close: Receiver<()>) -> Result<(), &'static str> {
    let mut config = substrate_service::Configuration::<(), _>::default_with_spec(make_chainspec());
    config.rpc_ws = Some(addr);
    config.rpc_ws_max_connections = Some(1);
    config.rpc_cors = None; // all connections are allowed
    config.database_path = store.join("database");
    config.keystore_path = store.join("keystore");

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

type OpaqueBlock = generic::Block<
    generic::Header<u32, sr_primitives::traits::BlakeTwo256>,
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
    .with_select_chain(|_config, backend| Ok(LongestChain::new(backend.clone())))?
    .with_transaction_pool(|config, client| Ok(Pool::new(config, ChainApi::new(client))))?
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
                InherentDataProviders::new(),
                Some(transaction_pool),
            )?;

        Ok(import_queue)
    })?
    .with_network_protocol(|_| Ok(NodeProtocol::new()))?
    .with_finality_proof_provider(|client, backend| {
        Ok(Arc::new(FinalityProofProvider::new(backend, client)))
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
fn timeout<T, Fo: Future<Item = T, Error = ()>>(
    timeout: Duration,
    f: impl Fn() -> Fo,
) -> impl Future<Item = T, Error = Duration> {
    use futures::future::Loop;

    let start = Instant::now();
    let end = start + timeout;

    futures::future::loop_fn((start, end), move |(start, end)| {
        f().then(move |res| {
            if let Ok(t) = res {
                return ok(Loop::Break(t));
            }

            if Instant::now() > end {
                return err(Instant::now() - start);
            }

            ok(Loop::Continue((start, end)))
        })
    })
}

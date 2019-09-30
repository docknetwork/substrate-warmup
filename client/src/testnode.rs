use crate::method::{Method, StateGetMetadata};
use crate::remote_call;
use futures::stream::Stream;
use futures::sync::mpsc::Receiver;
use futures::Future;
use substrate_warmup_chaingen::chain_spec;
use tempdir::TempDir;
use tokio::runtime::Builder as RuntimeBuilder;

pub struct RunningFullNode {
    url: String,
    chain_store: TempDir,
}

impl RunningFullNode {
    pub fn new() -> Self {
        let ret = Self {
            url: "ws://127.0.0.1:9944".to_string(),
            chain_store: TempDir::new("").unwrap(),
        };

        // check if local full-node is running
        ret.remote_call::<StateGetMetadata>([]).unwrap();

        // yep, it's running
        ret
    }

    pub fn remote_call<M: Method>(&self, arg: M::Args) -> Result<M::Return, &'static str> {
        remote_call::<M>(&self.url, arg)
    }
}

impl Drop for RunningFullNode {
    fn drop(&mut self) {
        match self.remote_call::<StateGetMetadata>([]) {
            Err(err) => {
                eprintln!("Full node stopped functioning during a test. Odd.. It was working");
                eprintln!("when the test started. A call to the StateGetMetadata method");
                eprintln!("resulted in the error: \"{}\"", err);
                if !std::thread::panicking() {
                    panic!()
                }
            }
            Ok(_) => {}
        }
    }
}

// run node until 'close' receives an item
fn run_node(url: String, chain_store: &TempDir, close: Receiver<()>) -> Result<(), &'static str> {
    let mut runtime = RuntimeBuilder::new()
        .name_prefix("main-tokio-")
        .build()
        .map_err(|e| "failed to build tokio runtime")?;
    let chainspec = chain_spec::Chain::Ved.generate_chainspec();
    let config = substrate_service::Configuration::<(), _>::default_with_spec(chainspec);
    // let node =
    //     substrate_service::NewService::new_full(config).map_err(|e| "couldn't create full node")?;
    let node = {
        substrate_service::ServiceBuilder::new_full(config)
            .map_err(|e| "couldn't create full node")?
            .with_select_chain(|_config, backend| {
                Ok(substrate_client::LongestChain::new(backend.clone()))
            })
            .map_err(|e| "couldn't create full node")?
            .with_transaction_pool(|config, client| {
                Ok(transaction_pool::txpool::Pool::new(
                    config,
                    transaction_pool::ChainApi::new(client),
                ))
            })
            .map_err(|e| "couldn't create full node")?
            .with_import_queue(|_config, client, mut select_chain, _transaction_pool| {
                let select_chain = select_chain
                    .take()
                    .ok_or_else(|| substrate_service::Error::SelectChainRequired)?;
                let (grandpa_block_import, grandpa_link) =
                    substrate_finality_grandpa::block_import::<
                        _,
                        _,
                        _,
                        node_template_runtime::RuntimeApi,
                        _,
                        _,
                    >(client.clone(), &*client, select_chain)
                    .map_err(|e| "couldn't create full node")?;
                let justification_import = grandpa_block_import.clone();

                let (babe_block_import, babe_link) = substrate_consensus_babe::block_import(
                    substrate_consensus_babe::Config::get_or_compute(&*client)?,
                    grandpa_block_import,
                    client.clone(),
                    client.clone(),
                )
                .map_err(|e| "couldn't create full node")?;

                let import_queue = substrate_consensus_babe::import_queue(
                    babe_link.clone(),
                    babe_block_import.clone(),
                    Some(Box::new(justification_import)),
                    None,
                    client.clone(),
                    client,
                    substrate_inherents::InherentDataProviders::new(),
                )
                .map_err(|e| "couldn't create full node")?;

                Ok(import_queue)
            })
            .map_err(|e| "couldn't create full node")?
            .build()
            .map_err(|e| "couldn't create full node")?
    };
    runtime.block_on(Stream::into_future(close).map(|_| ()));
    unimplemented!()
}

// fn service_from_config() -> Result<
//     NewService<
//         TBl,
//         Client<TBackend, TExec, TBl, TRtApi>,
//         TSc,
//         NetworkStatus<TBl>,
//         NetworkService<TBl, TNetP, <TBl as BlockT>::Hash>,
//         TransactionPool<TExPoolApi>,
//         OffchainWorkers<Client<TBackend, TExec, TBl, TRtApi>, TBackend::OffchainStorage, TBl>,
//     >,
//     Error,
// > {

// }

//! Service and ServiceFactory implementation. Specialized wrapper over Substrate service.

use futures::prelude::*;
use log::info;
use runtime::{self, opaque::Block, GenesisConfig, RuntimeApi, WASM_BINARY};
use std::sync::Arc;
use substrate_basic_authorship::ProposerFactory;
use substrate_client::{self as client, LongestChain};
use substrate_consensus_aura::{import_queue, start_aura, AuraImportQueue, SlotDuration};
use substrate_inherents::InherentDataProviders;
use substrate_network::config::DummyFinalityProofRequestBuilder;
use substrate_primitives::{ed25519::Pair, Pair as PairT};
use substrate_service::construct_service_factory;
use substrate_service::{
    error::Error as ServiceError, FactoryFullConfiguration, FullBackend, FullClient,
    FullComponents, FullExecutor, LightBackend, LightClient, LightComponents, LightExecutor,
};
use substrate_transaction_pool::{self, txpool::Pool as TransactionPool};

pub use substrate_executor::{
    with_native_environment, Blake2Hasher, Externalities, NativeExecutor, NativeVersion,
};

pub struct Executor;

impl substrate_executor::NativeExecutionDispatch for Executor {
    fn native_equivalent() -> &'static [u8] {
        WASM_BINARY
    }

    fn dispatch(
        ext: &mut Externalities<Blake2Hasher>,
        method: &str,
        data: &[u8],
    ) -> substrate_executor::error::Result<Vec<u8>> {
        with_native_environment(ext, || runtime::api::dispatch(method, data))?
            .ok_or_else(|| substrate_executor::error::Error::MethodNotFound(method.to_owned()))
    }

    fn native_version() -> NativeVersion {
        runtime::native_version()
    }

    fn new(default_heap_pages: Option<u64>) -> NativeExecutor<Self> {
        NativeExecutor::new(default_heap_pages)
    }
}

#[derive(Default)]
pub struct NodeConfig {
    inherent_data_providers: InherentDataProviders,
}

pub struct NodeProtocol;

impl NodeProtocol {
    pub fn new() -> Self {
        NodeProtocol
    }
}

impl substrate_network::specialization::NetworkSpecialization<Block> for NodeProtocol {
    fn status(&self) -> Vec<u8> {
        Vec::new()
    }

    fn on_connect(
        &mut self,
        _ctx: &mut substrate_network::Context<Block>,
        _who: substrate_network::PeerId,
        _status: substrate_network::StatusMessage<Block>,
    ) {
    }

    fn on_disconnect(
        &mut self,
        _ctx: &mut substrate_network::Context<Block>,
        _who: substrate_network::PeerId,
    ) {
    }

    fn on_message(
        &mut self,
        _ctx: &mut substrate_network::Context<Block>,
        _who: substrate_network::PeerId,
        _message: Vec<u8>,
    ) {
    }

    fn on_event(&mut self, _event: substrate_network::specialization::Event) {}

    fn on_abort(&mut self) {}

    fn maintain_peers(&mut self, _ctx: &mut substrate_network::Context<Block>) {}

    fn on_block_imported(
        &mut self,
        _ctx: &mut substrate_network::Context<Block>,
        _hash: <Block as substrate_network::BlockT>::Hash,
        _header: &<Block as substrate_network::BlockT>::Header,
    ) {
    }
}

construct_service_factory! {
    struct Factory {
        Block = Block,
        RuntimeApi = RuntimeApi,
        NetworkProtocol = NodeProtocol { |config| Ok(NodeProtocol::new()) },
        RuntimeDispatch = Executor,
        FullTransactionPoolApi = substrate_transaction_pool::ChainApi<
            client::Client<FullBackend<Self>, FullExecutor<Self>, Block, RuntimeApi>,
            Block
        > {
            |config, client| Ok(TransactionPool::new(
                config,
                substrate_transaction_pool::ChainApi::new(client),
            ))
        },
        LightTransactionPoolApi = substrate_transaction_pool::ChainApi<
            client::Client<LightBackend<Self>, LightExecutor<Self>, Block, RuntimeApi>,
            Block,
        > {
            |config, client| Ok(TransactionPool::new(
                config,
                substrate_transaction_pool::ChainApi::new(client),
            ))
        },
        Genesis = GenesisConfig,
        Configuration = NodeConfig,
        FullService = FullComponents<Self>
            { |config: FactoryFullConfiguration<Self>|
                FullComponents::<Factory>::new(config)
            },
        AuthoritySetup = {
            |service: Self::FullService| {
                if let Some(key) = service.authority_key::<Pair>() {
                    info!("Using authority key {}", key.public());
                    let proposer = Arc::new(ProposerFactory {
                        client: service.client(),
                        transaction_pool: service.transaction_pool(),
                    });
                    let client = service.client();
                    let select_chain = service.select_chain()
                        .ok_or_else(|| ServiceError::SelectChainRequired)?;
                    let aura = start_aura(
                        SlotDuration::get_or_compute(&*client)?,
                        Arc::new(key),
                        client.clone(),
                        select_chain,
                        client,
                        proposer,
                        service.network(),
                        service.config.custom.inherent_data_providers.clone(),
                        service.config.force_authoring,
                    )?;
                    service.spawn_task(Box::new(aura.select(service.on_exit()).then(|_| Ok(()))));
                }

                Ok(service)
            }
        },
        LightService = LightComponents<Self>
            { |config| <LightComponents<Factory>>::new(config) },
        FullImportQueue = AuraImportQueue<
            Self::Block,
        >
            { |config: &mut FactoryFullConfiguration<Self> , client: Arc<FullClient<Self>>, _select_chain: Self::SelectChain| {
                    import_queue::<_, _, Pair>(
                        SlotDuration::get_or_compute(&*client)?,
                        Box::new(client.clone()),
                        None,
                        None,
                        client,
                        config.custom.inherent_data_providers.clone(),
                    ).map_err(Into::into)
                }
            },
        LightImportQueue = AuraImportQueue<
            Self::Block,
        >
            { |config: &mut FactoryFullConfiguration<Self>, client: Arc<LightClient<Self>>| {
                    let fprb = Box::new(DummyFinalityProofRequestBuilder::default()) as Box<_>;
                    import_queue::<_, _, Pair>(
                        SlotDuration::get_or_compute(&*client)?,
                        Box::new(client.clone()),
                        None,
                        None,
                        client,
                        config.custom.inherent_data_providers.clone(),
                    ).map(|q| (q, fprb)).map_err(Into::into)
                }
            },
        SelectChain = LongestChain<FullBackend<Self>, Self::Block>
            { |config: &FactoryFullConfiguration<Self>, client: Arc<FullClient<Self>>| {
                #[allow(deprecated)]
                Ok(LongestChain::new(client.backend().clone()))
            }
        },
        FinalityProofProvider = { |_client: Arc<FullClient<Self>>| {
            Ok(None)
        }},
    }
}

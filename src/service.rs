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
use substrate_service::{
    error::Error as ServiceError, FactoryFullConfiguration, FullBackend, FullComponents,
    FullExecutor, LightBackend, LightComponents, LightExecutor,
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

pub struct Factory;

impl substrate_service::ServiceFactory for Factory {
    type Block = Block;
    type RuntimeApi = RuntimeApi;
    type NetworkProtocol = NodeProtocol;
    type RuntimeDispatch = Executor;
    type FullTransactionPoolApi = substrate_transaction_pool::ChainApi<
        client::Client<FullBackend<Self>, FullExecutor<Self>, Block, RuntimeApi>,
        Block,
    >;
    type LightTransactionPoolApi = substrate_transaction_pool::ChainApi<
        client::Client<LightBackend<Self>, LightExecutor<Self>, Block, RuntimeApi>,
        Block,
    >;
    type Genesis = GenesisConfig;
    type Configuration = NodeConfig;
    type FullService = FullComponents<Self>;
    type LightService = LightComponents<Self>;
    type FullImportQueue = AuraImportQueue<Self::Block>;
    type LightImportQueue = AuraImportQueue<Self::Block>;
    type SelectChain = LongestChain<FullBackend<Self>, Self::Block>;

    fn build_full_transaction_pool(
        config: substrate_service::TransactionPoolOptions,
        client: substrate_service::Arc<substrate_service::FullClient<Self>>,
    ) -> substrate_service::Result<
        substrate_service::TransactionPool<Self::FullTransactionPoolApi>,
        substrate_service::Error,
    > {
        Ok(TransactionPool::new(
            config,
            substrate_transaction_pool::ChainApi::new(client),
        ))
    }

    fn build_light_transaction_pool(
        config: substrate_service::TransactionPoolOptions,
        client: substrate_service::Arc<substrate_service::LightClient<Self>>,
    ) -> substrate_service::Result<
        substrate_service::TransactionPool<Self::LightTransactionPoolApi>,
        substrate_service::Error,
    > {
        Ok(TransactionPool::new(
            config,
            substrate_transaction_pool::ChainApi::new(client),
        ))
    }

    fn build_network_protocol(
        _config: &substrate_service::FactoryFullConfiguration<Self>,
    ) -> substrate_service::Result<Self::NetworkProtocol, substrate_service::Error> {
        Ok(NodeProtocol::new())
    }

    fn build_select_chain(
        _config: &mut substrate_service::FactoryFullConfiguration<Self>,
        client: Arc<substrate_service::FullClient<Self>>,
    ) -> substrate_service::Result<Self::SelectChain, substrate_service::Error> {
        // client.backend() is deprecated but at present there is no alternative
        // https://github.com/paritytech/substrate/issues/1134
        #[allow(deprecated)]
        Ok(LongestChain::new(client.backend().clone()))
    }

    fn build_full_import_queue(
        config: &mut substrate_service::FactoryFullConfiguration<Self>,
        client: substrate_service::Arc<substrate_service::FullClient<Self>>,
        _select_chain: Self::SelectChain,
    ) -> substrate_service::Result<Self::FullImportQueue, substrate_service::Error> {
        import_queue::<_, _, Pair>(
            SlotDuration::get_or_compute(&*client)?,
            Box::new(client.clone()),
            None,
            None,
            client,
            config.custom.inherent_data_providers.clone(),
        )
        .map_err(Into::into)
    }

    fn build_light_import_queue(
        config: &mut FactoryFullConfiguration<Self>,
        client: Arc<substrate_service::LightClient<Self>>,
    ) -> Result<
        (
            Self::LightImportQueue,
            substrate_service::BoxFinalityProofRequestBuilder<Block>,
        ),
        substrate_service::Error,
    > {
        let fprb = Box::new(DummyFinalityProofRequestBuilder::default()) as Box<_>;
        import_queue::<_, _, Pair>(
            SlotDuration::get_or_compute(&*client)?,
            Box::new(client.clone()),
            None,
            None,
            client,
            config.custom.inherent_data_providers.clone(),
        )
        .map(|q| (q, fprb))
        .map_err(Into::into)
    }

    fn build_finality_proof_provider(
        _client: Arc<substrate_service::FullClient<Self>>,
    ) -> Result<
        Option<Arc<substrate_service::FinalityProofProvider<Self::Block>>>,
        substrate_service::Error,
    > {
        Ok(None)
    }

    fn new_light(
        config: substrate_service::FactoryFullConfiguration<Self>,
    ) -> substrate_service::Result<Self::LightService, substrate_service::Error> {
        <LightComponents<Factory>>::new(config)
    }

    fn new_full(
        config: substrate_service::FactoryFullConfiguration<Self>,
    ) -> Result<Self::FullService, substrate_service::Error> {
        FullComponents::<Factory>::new(config).and_then(|service: Self::FullService| {
            if let Some(key) = service.authority_key::<Pair>() {
                info!("Using authority key {}", key.public());
                let proposer = Arc::new(ProposerFactory {
                    client: service.client(),
                    transaction_pool: service.transaction_pool(),
                });
                let client = service.client();
                let select_chain = service
                    .select_chain()
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
        })
    }
}

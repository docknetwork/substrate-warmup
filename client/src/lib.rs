pub mod method;
pub mod storage;
pub mod testnode;
pub mod wrapped_client;

use crate::wrapped_client::WrappedClient;
use futures::Future;
use jsonrpc_client_transports::{transports, RpcChannel, RpcError};
use serde::de::DeserializeOwned;
use srml_system::Trait as System;
use substrate_rpc_api::{
    author::AuthorClient, chain::ChainClient, state::StateClient, system::SystemClient,
};
use url::Url;

pub struct NodeRpcClient<R: System>
where
    R::Header: DeserializeOwned,
{
    state: StateClient<R::Hash>,
    // This second AuthorClient template parameter may be wrong. I don't know what the difference
    // between a Hash and a BlockHash is.
    author: AuthorClient<R::Hash, R::Hash>,
    // SignedBlock as a Vec<u8> is definitely incorrect
    chain: ChainClient<R::BlockNumber, R::Hash, R::Header, Vec<u8>>,
    system: SystemClient<R::Hash, R::BlockNumber>,
}

impl<R: System> NodeRpcClient<R>
where
    R::Header: DeserializeOwned,
{
    /// # Panics
    ///
    /// Panics if not called within a tokio runtime
    ///
    /// # Reliance on Tokio
    ///
    /// This relies on the jsonrpc_client_transports websocket transport, which assumes a tokio
    /// runtime to exist for the entire life of the websocket.
    ///
    /// The client returned by this function will not function after its runtime stops.
    // This limitation may be fixed later if we use raw tcp instead of websockets.
    pub fn connect(websocket_url: &Url) -> impl Future<Item = Self, Error = RpcError> {
        transports::ws::connect::<RpcChannel>(websocket_url).map(|channel| NodeRpcClient {
            state: StateClient::new(channel.clone()),
            author: AuthorClient::new(channel.clone()),
            chain: ChainClient::new(channel.clone()),
            system: SystemClient::new(channel),
        })
    }

    pub fn author(&self) -> WrappedClient<&AuthorClient<R::Hash, R::Hash>> {
        WrappedClient(&self.author)
    }

    pub fn chain(
        &self,
    ) -> WrappedClient<&ChainClient<R::BlockNumber, R::Hash, R::Header, Vec<u8>>> {
        WrappedClient(&self.chain)
    }

    /// ```no_run
    /// use client::NodeRpcClient;
    /// use node_template_runtime::Runtime;
    /// use srml_metadata::RuntimeMetadataPrefixed;
    /// use srml_system::Trait as System;
    /// let client: NodeRpcClient<Runtime> = unimplemented!();
    /// let metadata = client.state().metadata(None); // returns Future<Item = RuntimeMetadataPrefixed, ..>
    /// let raw_meta = client.state().0.metadata(None); // returns Future<Item = Bytes, ..>
    /// ```
    pub fn state(&self) -> WrappedClient<&StateClient<R::Hash>> {
        WrappedClient(&self.state)
    }

    pub fn system(&self) -> WrappedClient<&SystemClient<R::Hash, R::BlockNumber>> {
        WrappedClient(&self.system)
    }

    // Wrappers should be typed. No returning Bytes.
    // StorageKey should be replaced with something meaningful.
    // Test all implemented methods.
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;
    use node_template_runtime::Runtime;
    use testnode::RunningFullNode;

    /// Perform test within a tokio runtime
    fn with_runtime<Fut>(fut: Fut) -> Result<Fut::Item, Fut::Error>
    where
        Fut: Future + Send + 'static,
        Fut::Error: Send + 'static,
        Fut::Item: Send + 'static,
    {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(fut)
    }

    /// spawns test node and keeps it alive long enough to perform test
    fn with_node<T, Fut>(
        f: impl Fn(&NodeRpcClient<Runtime>) -> Fut,
    ) -> impl Future<Item = T, Error = failure::Error>
    where
        Fut: Future<Item = T, Error = failure::Error>,
    {
        RunningFullNode::new()
            .map_err(failure::err_msg)
            .and_then(move |node| {
                f(node.client()).map(move |t| {
                    let _ = node; // keeps node alive until t is computed
                    t
                })
            })
    }

    #[test]
    /// Spawing a test node takes a long time (around ten seconds) so we usne the same node for all
    /// client tests.
    /// All client tests are asyncronous so they can be run at the same time, over the same
    /// websocket.
    fn all_client_tests() {
        let fut = with_node(|client| {
            let todo: Vec<_> = ALL_TESTS.iter().map(|f| f(client)).collect();
            futures::future::join_all(todo)
        });
        with_runtime(fut).unwrap();
    }

    type Client = NodeRpcClient<Runtime>;
    type TestFut = Box<dyn Future<Item = (), Error = failure::Error> + Send>;
    type ClientTest = fn(client: &Client) -> TestFut;

    const ALL_TESTS: &[ClientTest] = &[metadata, metadata_manual];

    fn metadata(client: &Client) -> TestFut {
        let ret = client.state().metadata(None);
        Box::new(ret.map(|_| ()).map_err(Into::into))
    }

    fn metadata_manual(client: &Client) -> TestFut {
        use parity_scale_codec::DecodeAll;
        use srml_metadata::RuntimeMetadataPrefixed;
        use substrate_primitives::OpaqueMetadata;

        // This method requires two steps to decode. The original runtime method returns a
        // RuntimeMetadataPrefixed encoded and wraped as OpaqueMetadata.
        // Its not possible to encode as OpaqueMetadata then decode as RuntimeMetadataPrefixed
        // because the encoded version of OpaqueMetadata adds a length prefix.
        //
        // encoded = encode(bytesize(encode(metadata))) ++ encode(metadata)
        declare_method!(GetMeta, "Metadata_metadata", (), OpaqueMetadata);

        let ret = client
            .state()
            .call::<GetMeta>((), None) // get OpaqueMetadata
            .map_err(Into::into)
            .and_then(|opaque| -> Result<RuntimeMetadataPrefixed, _> {
                DecodeAll::decode_all(&opaque).map_err(Into::into) // parse as RuntimeMetadataPrefixed
            });

        Box::new(ret.map(|_| ()))
    }
}

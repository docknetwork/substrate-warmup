pub mod testnode;
use core::any::Any;
use futures::Future;
use jsonrpc_client_transports::{transports, RpcChannel, RpcError};
use parity_scale_codec::DecodeAll;
use serde::de::DeserializeOwned;
use srml_metadata::RuntimeMetadataPrefixed;
use srml_system::Trait as System;
use substrate_primitives::Bytes;
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
    // SignedBlock as a Vec<u8> is definately incorrect
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
    pub fn connect(websocket_url: &Url) -> impl Future<Item = Self, Error = RpcError> {
        transports::ws::connect::<RpcChannel>(websocket_url).map(|channel| NodeRpcClient {
            state: StateClient::new(channel.clone()),
            author: AuthorClient::new(channel.clone()),
            chain: ChainClient::new(channel.clone()),
            system: SystemClient::new(channel),
        })
    }

    /// When block index is None, the most recent block is assumed.
    ///
    /// # Panics
    ///
    /// Panics if not called within a tokio runtime
    pub fn state_metadata(
        &self,
        block_index: Option<R::Hash>,
    ) -> impl Future<Item = RuntimeMetadataPrefixed, Error = RpcError> {
        self.state
            .metadata(block_index)
            .and_then(|bytes| -> Result<RuntimeMetadataPrefixed, _> {
                DecodeAll::decode_all(&bytes).map_err(|codec_err| {
                    let bytes: &[u8] = bytes.as_ref();
                    RpcError::ParseError(
                        format!("failure decoding scale {}", hex::encode(bytes)),
                        codec_err.into(),
                    )
                })
            })
    }

    /// TODO:
    ///   what does name mean?
    ///   what does bytes represent?
    ///   what does the return value represent?
    ///   test
    pub fn state_call(
        &self,
        _name: String,
        _bytes: Bytes,
        _block_index: Option<R::Hash>,
    ) -> impl Future<Item = Bytes, Error = RpcError> {
        futures::future::ok(unimplemented!())
    }

    // TODO implement wrapper for all methods from StateClient, AuthorClient, ChainClient, and
    // SystemClient.
    // Wrappers should be typed. No returning Bytes.
    // StorageKey should be replaced with something meaningful.
    // Test all implemented methods.
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Fail;
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

    const ALL_TESTS: &[ClientTest] = &[metadata];

    fn metadata(client: &Client) -> TestFut {
        let ret = client.state_metadata(None);
        Box::new(ret.map(|_| ()).map_err(Into::into))
    }
}

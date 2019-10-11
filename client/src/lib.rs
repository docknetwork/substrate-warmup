pub mod testnode;

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;
    use parity_scale_codec::DecodeAll;
    use substrate_rpc_api::state::StateClient;
    use testnode::RunningFullNode;

    type Hash = <node_template_runtime::Runtime as srml_system::Trait>::Hash;

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

    fn new_node() -> RunningFullNode {
        with_runtime(RunningFullNode::new()).unwrap()
    }

    #[test]
    fn get_metadata() {
        use srml_metadata::RuntimeMetadataPrefixed;
        let node = new_node();

        let res = node
            .client_channel()
            .and_then(|rpc_channel| StateClient::<Hash>::new(rpc_channel).metadata(None))
            .map(|dang_bytes| -> Result<RuntimeMetadataPrefixed, _> {
                DecodeAll::decode_all(&dang_bytes)
            });

        with_runtime(res).unwrap().unwrap();
    }
}

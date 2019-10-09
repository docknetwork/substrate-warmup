pub mod method;
pub mod remote_call;
pub mod serde_as_scale;

#[cfg(test)]
mod testnode;

#[cfg(test)]
mod tests {
    use super::*;
    use method::StateGetMetadata;
    use testnode::RunningFullNode;

    #[test]
    fn g_metadata() {
        RunningFullNode::new()
            .remote_call::<StateGetMetadata>([])
            .unwrap();
    }

    #[test]
    fn metadata_same() {
        // Get metadata from server, assert it is equal to metadata exported by runtime.
        let remote_metadata = RunningFullNode::new()
            .remote_call::<StateGetMetadata>([])
            .unwrap();
        let expected_metadata = node_template_runtime::Runtime::metadata();
        assert_eq!(remote_metadata, expected_metadata);
    }
}

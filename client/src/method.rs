pub trait Method {
    /// unique static name passed as the jsonrpc "method" field of the jsonrpc call
    const NAME: &'static str;

    /// serialized to the "params" field of the jsonrpc call
    type Args: serde::Serialize;

    /// serialized to the "result" field of the jsonrpc response
    type Return: parity_scale_codec::Decode;
}

pub struct StateGetMetadata;

impl Method for StateGetMetadata {
    const NAME: &'static str = "state_getMetadata";
    type Args = NoArgs;
    type Return = srml_metadata::RuntimeMetadataPrefixed;
}

pub type NoArgs = [bool; 0];

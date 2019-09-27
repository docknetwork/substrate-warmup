use serde::Serialize;

pub trait Method {
    /// unique static name passed as the jsonrpc "method" field of the jsonrpc call
    const NAME: &'static str;

    /// serialized to the "params" field of the jsonrpc call
    type Args: Serialize;

    /// serialized to the "result" field of the jsonrpc response
    type Return: parity_scale_codec::Decode;
}

pub struct StateGetMetadata;
impl Method for StateGetMetadata {
    const NAME: &'static str = "state_getMetadata";
    /// serializes to json as an empty list.
    type Args = [bool; 0];
    type Return = srml_metadata::RuntimeMetadataPrefixed;
}

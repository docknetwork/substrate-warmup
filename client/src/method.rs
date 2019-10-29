use parity_scale_codec::{Decode, Encode};

/// A method callable within a substrate Runtime.
///
/// ```
/// use substrate_primitives::OpaqueMetadata;
/// use client::method::Method;
///
/// pub struct GetMeta;
/// impl Method for GetMeta {
///     const NAME: &'static str = "Metadata_metadata";
///     type Args = ();
///     type Return = OpaqueMetadata;
/// }
/// ```
///
/// A macro is provided for convenience. The following example is equivalent to the previous.
///
/// ```
/// use substrate_primitives::OpaqueMetadata;
/// use client::declare_method;
///
/// declare_method!(GetMeta, "Metadata_metadata", (), OpaqueMetadata);
/// ```
pub trait Method {
    /// unique static name passed as the jsonrpc "method" field of the jsonrpc call
    const NAME: &'static str;

    /// serialized to the "params" field of the jsonrpc call
    type Args: Encode;

    /// serialized to the "result" field of the jsonrpc response
    type Return: Decode;
}

/// ```
/// use substrate_primitives::OpaqueMetadata;
/// use client::declare_method;
///
/// declare_method!(
///     GetMeta,             // name of the type that will represent the method
///     "Metadata_metadata", // NAME
///     (),                  // Args
///     OpaqueMetadata       // Return
/// );
/// ```
#[macro_export]
macro_rules! declare_method {
    ($struct_name:ident, $method_name:expr, $args:ty, $return:ty) => {
        pub struct $struct_name;

        impl $crate::method::Method for $struct_name {
            const NAME: &'static str = $method_name;
            type Args = $args;
            type Return = $return;
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use parity_scale_codec::DecodeAll;

    /// the unit type: (), encodes to and decodes from an empty byte array
    #[test]
    fn unit_codec() {
        assert_eq!(&().encode(), &[0u8; 0]);
        assert_eq!(DecodeAll::decode_all(&[]), Ok(()));
    }
}

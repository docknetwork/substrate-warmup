use parity_scale_codec::{Decode, DecodeAll, Encode};
use substrate_primitives::storage::{StorageData, StorageKey};

/// A query on runtime storage
/// The type implementing StorageQuery is the key some Value in runtime storage.
pub trait StorageQuery: Encode {
    /// The result of this query
    type Value: Decode;

    fn as_key(&self) -> StorageKey {
        unimplemented!()
    }

    fn decode_value(raw: &StorageData) -> Result<Self::Value, parity_scale_codec::Error> {
        DecodeAll::decode_all(&raw.0)
    }
}

/// Potentially useful: srml_support::storage::generator

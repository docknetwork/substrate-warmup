use parity_scale_codec::{DecodeAll, Encode};

/// A query on runtime storage
/// The type iplementing StorageQuery is the key some Value stored in runtime storage.
pub trait StorageQuery: Encode {
    /// The result of this query
    type Value: DecodeAll;
}

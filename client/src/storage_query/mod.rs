mod augment_clap;
mod double_map;
mod linked_map;
mod map;
mod value;

use parity_scale_codec::DecodeAll;
use serde::Serialize;
use serde_json::Value;
use substrate_primitives_storage::{StorageData, StorageKey};

pub use augment_clap::AugmentClap;
pub use map::MapQuery;
pub use value::ValueQuery;

pub trait StorageQuery {
    fn to_raw_key(&self) -> StorageKey;

    /// May panic if in-memory serialization fails.
    fn raw_scale_to_json(&self, raw: StorageData) -> Result<Value, parity_scale_codec::Error>;
}

/// Panics if in-memory serialization fails.
fn raw_scale_to_json<T: DecodeAll + Serialize>(
    raw: StorageData,
) -> Result<Value, parity_scale_codec::Error> {
    let ret = T::decode_all(&raw.0)?;
    let json = serde_json::to_value(&ret).expect("error encoding storage value as json");
    Ok(json)
}

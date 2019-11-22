mod augment_clap;
mod double_map;
mod linked_map;
mod map;
mod value;

use super::json::Json;
use parity_scale_codec::DecodeAll;
use serde::Serialize;
use substrate_primitives_storage::{StorageData, StorageKey};

pub use augment_clap::AugmentClap;
pub use map::MapQuery;
pub use value::ValueQuery;

pub trait StorageQuery {
    fn to_raw_key(&self) -> StorageKey;

    /// May panic if in-memory serialization fails.
    fn raw_scale_to_json(&self, raw: StorageData) -> Result<Json, parity_scale_codec::Error>;
}

/// Panics if in-memory serialization fails.
fn raw_scale_to_json<T: DecodeAll + Serialize>(
    raw: StorageData,
) -> Result<Json, parity_scale_codec::Error> {
    let ret = T::decode_all(&raw.0)?;
    Ok(Json::create(&ret).unwrap())
}

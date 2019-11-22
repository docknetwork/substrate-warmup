mod double_map;
mod linked_map;
mod map;
mod value;

use parity_scale_codec::DecodeAll;
use serde::Serialize;
use serde_json::Value;
use substrate_primitives_storage::{StorageData, StorageKey};

pub use value::ValueQuery;

pub trait StorageQuery {
    type Return;

    fn to_raw_key(&self) -> StorageKey;

    /// Panics if in-memory serialization fails.
    fn raw_scale_to_json(&self, raw: StorageData) -> Result<Value, parity_scale_codec::Error>
    where
        Self::Return: DecodeAll + Serialize,
    {
        let ret = Self::Return::decode_all(&raw.0)?;
        let json = serde_json::to_value(&ret).expect("error encoding storage value as json");
        Ok(json)
    }
}

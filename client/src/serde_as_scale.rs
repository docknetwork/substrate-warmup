//! Declares a wrapper type for implementers of [`parity_scale_codec::Encode`].
//! The wrapper type implements [`serde::Serialize`] using the [`parity_scale_codec::Encode`]
//! implementation.
//!
//! ```
//! use crate::client::serde_as_scale::SerdeAsScale;
//!
//! let a = SerdeAsScale(true);
//! let ja = serde_json::to_string(&a).unwrap();
//! assert_eq!(ja, "\"0x01\"");
//! ```
//!
//! ```
//! # use crate::client::serde_as_scale::SerdeAsScale;
//! #
//! let a = SerdeAsScale(true);
//! let b = SerdeAsScale(&false);
//! assert_eq!(serde_json::to_string(&a).unwrap(), "\"0x01\"");
//! assert_eq!(serde_json::to_string(&b).unwrap(), "\"0x00\"");
//! ```

use core::fmt;
use parity_scale_codec::{Decode, Encode};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct SerdeAsScale<T>(pub T);

impl<T: Encode> Serialize for SerdeAsScale<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // this funcion performs two allocations. We can optimize later using [collect_str]
        // (https://docs.rs/serde/1.0.101/serde/trait.Serializer.html#method.collect_str)
        let hexstr = self.0.using_encoded(|bytes| hex::encode(bytes));
        serializer.serialize_str(&format!("0x{}", hexstr))
    }
}

impl<'de, T: Decode> Deserialize<'de> for SerdeAsScale<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes: Vec<u8> = deserializer.deserialize_str(ScaleVisitor)?;
        let ret: T = parity_scale_codec::DecodeAll::decode_all(&bytes).map_err(|decode_err| {
            de::Error::invalid_value(de::Unexpected::Other(decode_err.what()), &decode_err.what())
        })?;
        Ok(SerdeAsScale(ret))
    }
}

struct ScaleVisitor;

impl<'de> de::Visitor<'de> for ScaleVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a '0x' prefixed string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if !s.starts_with("0x") {
            return Err(de::Error::invalid_value(
                de::Unexpected::Str(s),
                &"'0x' prefixed string",
            ));
        }
        hex::decode(&s[2..]).map_err(|decode_err| match decode_err {
            hex::FromHexError::InvalidHexCharacter { c, .. } => {
                de::Error::invalid_value(de::Unexpected::Char(c), &"[0-9a-fA-F]")
            }
            hex::FromHexError::InvalidStringLength => {
                panic!("invalid string length error shoulf not occur when calling hex::decode")
            }
            hex::FromHexError::OddLength => {
                de::Error::invalid_value(de::Unexpected::Str(s), &"hex string with even length")
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use de::DeserializeOwned;
    use serde_json::{json, Value};

    // deserialize json
    fn dej<T: DeserializeOwned>(j: Value) -> T {
        serde_json::from_value(j.clone()).unwrap()
    }

    // serialize json
    fn sej<T: Serialize>(t: T) -> Value {
        serde_json::to_value(&t).unwrap()
    }

    #[test]
    fn ser() {
        assert_eq!(sej(SerdeAsScale(true)), json!("0x01"));
        assert_eq!(sej(SerdeAsScale(false)), json!("0x00"));
    }

    #[test]
    fn deser() {
        assert_eq!(dej::<SerdeAsScale<bool>>(json!("0x01")), SerdeAsScale(true));
        assert_eq!(
            dej::<SerdeAsScale<bool>>(json!("0x00")),
            SerdeAsScale(false)
        );
    }
}

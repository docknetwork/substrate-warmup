//! serde_json::Value cannot serialize u128 by default. This is a workaround.
//! json is serialized to a String. An invaiant is upheld: the String is valid
//! json.

use serde::Serialize;

pub struct Json(String);

impl Json {
    pub fn create<T: Serialize>(src: &T) -> Result<Self, serde_json::Error> {
        serde_json::to_string(src).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

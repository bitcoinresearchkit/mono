#[cfg(feature = "schemars")]
use std::borrow::Cow;

use derive_more::{Deref, DerefMut};
#[cfg(feature = "schemars")]
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use serde_bytes as bytes;

#[cfg(feature = "storage")]
use vecdb::Bytes;

#[derive(
    Debug, Clone, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "storage", derive(Bytes))]
pub struct U8x65(#[serde(with = "bytes")] [u8; 65]);

#[cfg(feature = "schemars")]
impl JsonSchema for U8x65 {
    fn schema_name() -> Cow<'static, str> {
        "U8x65".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        Vec::<u8>::json_schema(generator)
    }
}

impl From<&[u8]> for U8x65 {
    #[inline]
    fn from(slice: &[u8]) -> Self {
        let mut arr = [0; 65];
        arr.copy_from_slice(slice);
        Self(arr)
    }
}

use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Result},
};

use derive_more::Deref;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

/// Fixed-size boolean value optimized for on-disk storage (stored as u8)
#[derive(
    Debug, Deref, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct StoredBool(u8);

impl JsonSchema for StoredBool {
    fn schema_name() -> Cow<'static, str> {
        "StoredBool".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        bool::json_schema(generator)
    }
}

impl StoredBool {
    pub const FALSE: Self = Self(0);
    pub const TRUE: Self = Self(1);

    pub fn is_true(&self) -> bool {
        *self == Self::TRUE
    }

    pub fn is_false(&self) -> bool {
        *self == Self::FALSE
    }
}

impl From<bool> for StoredBool {
    #[inline]
    fn from(value: bool) -> Self {
        if value { Self(1) } else { Self(0) }
    }
}

impl From<StoredBool> for usize {
    #[inline]
    fn from(value: StoredBool) -> Self {
        value.0 as usize
    }
}

impl StoredBool {
    fn index_name() -> &'static str {
        "bool"
    }
    fn index_aliases() -> &'static [&'static str] {
        &["bool"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for StoredBool {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Display for StoredBool {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.is_true() {
            f.write_str("true")
        } else {
            f.write_str("false")
        }
    }
}

#[cfg(feature = "storage")]
impl Formattable for StoredBool {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(if self.is_true() { b"true" } else { b"false" });
    }
}

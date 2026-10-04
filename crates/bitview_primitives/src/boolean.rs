use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Result},
};

use derive_more::Deref;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Yes or no (stored as u8; JSON `true`/`false`).
#[derive(
    Debug, Deref, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(from = "bool", into = "bool")]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Boolean(u8);

impl JsonSchema for Boolean {
    fn schema_name() -> Cow<'static, str> {
        "Boolean".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let mut schema = bool::json_schema(generator);
        schema.insert("description".into(), "Yes or no.".into());
        schema
    }
}

impl Boolean {
    pub const FALSE: Self = Self(0);
    const TRUE: Self = Self(1);

    pub fn is_true(&self) -> bool {
        *self == Self::TRUE
    }

    pub fn is_false(&self) -> bool {
        *self == Self::FALSE
    }
}

impl From<bool> for Boolean {
    #[inline]
    fn from(value: bool) -> Self {
        if value { Self(1) } else { Self(0) }
    }
}

impl From<Boolean> for bool {
    #[inline]
    fn from(value: Boolean) -> Self {
        value.is_true()
    }
}

impl From<Boolean> for usize {
    #[inline]
    fn from(value: Boolean) -> Self {
        value.0 as usize
    }
}

impl Display for Boolean {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.is_true() {
            f.write_str("true")
        } else {
            f.write_str("false")
        }
    }
}

#[cfg(feature = "storage")]
impl Formattable for Boolean {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(if self.is_true() { b"true" } else { b"false" });
    }
}

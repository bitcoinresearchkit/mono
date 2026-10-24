use std::borrow::Cow;

use derive_more::Deref;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};

/// How many results one response holds: 1 to 1,000. Parsing anything else fails, naming the
/// range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, Serialize, Deserialize)]
#[serde(try_from = "usize", into = "usize")]
pub struct PageSize(usize);

impl PageSize {
    pub const MIN: usize = 1;
    pub const MAX: usize = 1_000;

    /// For defaults; panics outside the range.
    pub const fn new(size: usize) -> Self {
        assert!(size >= Self::MIN && size <= Self::MAX);
        Self(size)
    }
}

impl TryFrom<usize> for PageSize {
    type Error = String;

    fn try_from(size: usize) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&size) {
            Ok(Self(size))
        } else {
            Err(format!(
                "{size} is out of range, expected {} to {}",
                Self::MIN,
                Self::MAX
            ))
        }
    }
}

impl From<PageSize> for usize {
    fn from(size: PageSize) -> Self {
        size.0
    }
}

impl JsonSchema for PageSize {
    fn schema_name() -> Cow<'static, str> {
        "PageSize".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "integer",
            "format": "uint",
            "minimum": Self::MIN,
            "maximum": Self::MAX,
        })
    }

    /// A bounded integer, not a type of its own: clients take a plain number.
    fn inline_schema() -> bool {
        true
    }
}

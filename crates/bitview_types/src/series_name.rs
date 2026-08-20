use std::{
    borrow::Cow,
    fmt::{Display, Formatter, Result},
};

use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Series name
#[derive(Debug, Clone, Deref, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(
    with = "String",
    example = &"price_close",
    example = &"market_cap",
    example = &"realized_price"
)]
pub struct SeriesName(String);

impl SeriesName {
    /// Lookup key: `-` → `_`, lowercased. Borrows when already normalized.
    pub fn normalize(&self) -> Cow<'_, str> {
        if self.0.bytes().any(|b| b == b'-' || b.is_ascii_uppercase()) {
            Cow::Owned(self.0.replace('-', "_").to_lowercase())
        } else {
            Cow::Borrowed(&self.0)
        }
    }
}

impl From<String> for SeriesName {
    #[inline]
    fn from(series: String) -> Self {
        Self(series)
    }
}

impl From<&str> for SeriesName {
    #[inline]
    fn from(series: &str) -> Self {
        Self(series.to_owned())
    }
}

impl Display for SeriesName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

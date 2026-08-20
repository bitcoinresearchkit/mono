use std::fmt;

use derive_more::Deref;
use schemars::JsonSchema;
use serde::Deserialize;

/// Maximum number of results to return. Defaults to 100 if not specified.
#[derive(Debug, Clone, Copy, Deref, Deserialize, JsonSchema)]
#[serde(transparent)]
#[allow(clippy::duplicated_attributes)]
#[schemars(default, example = 1, example = 10, example = 100)]
pub struct Limit(usize);

impl Limit {
    pub const MIN: Self = Self(1);
    pub const DEFAULT: Self = Self(100);

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Default for Limit {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for Limit {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

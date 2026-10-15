use std::fmt;

use schemars::JsonSchema;
use serde::Deserialize;

/// Output format for API responses
#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    #[default]
    #[serde(alias = "json")]
    JSON,
    #[serde(alias = "csv")]
    CSV,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::JSON => write!(f, "json"),
            Format::CSV => write!(f, "csv"),
        }
    }
}

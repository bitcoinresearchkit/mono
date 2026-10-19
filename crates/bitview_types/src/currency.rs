use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A quote currency prices are served in: USD only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Currency {
    USD,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("USD")
    }
}

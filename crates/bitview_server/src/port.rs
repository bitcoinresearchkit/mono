use std::{fmt, num::ParseIntError, str::FromStr};

use derive_more::Deref;
use serde::{Deserialize, Serialize};

/// Server port. Defaults to 3110.
#[derive(Debug, Clone, Copy, Deref, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Port(u16);

impl Port {
    pub const DEFAULT: Self = Self(3110);
}

impl Default for Port {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u16> for Port {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Port> for u16 {
    fn from(value: Port) -> Self {
        value.0
    }
}

impl FromStr for Port {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u16>().map(Self)
    }
}

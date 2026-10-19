use derive_more::{Deref, Display, From, FromStr, Into};
use serde::{Deserialize, Serialize};

/// Server port. Defaults to 3110.
#[derive(
    Debug,
    Clone,
    Copy,
    Deref,
    Display,
    From,
    Into,
    FromStr,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
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

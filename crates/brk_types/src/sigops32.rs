use derive_more::{From, Into};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::Pco;

/// A transaction's BIP-141 signature-operation cost in 32 bits: the storage form of `SigOps`.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    From,
    Into,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[serde(transparent)]
pub struct SigOps32(u32);

impl SigOps32 {
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

impl From<usize> for SigOps32 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

crate::impl_int_format!(SigOps32);

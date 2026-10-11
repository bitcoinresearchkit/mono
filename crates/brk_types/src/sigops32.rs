#[cfg(feature = "storage")]
use itoa::Buffer;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// A transaction's BIP-141 signature-operation cost in 32 bits: the storage form of `SigOps`.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
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

impl From<u32> for SigOps32 {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<usize> for SigOps32 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl From<SigOps32> for u32 {
    #[inline]
    fn from(value: SigOps32) -> Self {
        value.0
    }
}

#[cfg(feature = "storage")]
impl Formattable for SigOps32 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

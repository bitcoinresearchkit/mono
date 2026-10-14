#[cfg(feature = "storage")]
use itoa::Buffer;
use std::fmt::{Display, Formatter, Result};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::Formattable;

/// Index of the output being spent in the previous transaction (coinbase inputs: 4294967295)
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(example = &0, example = &1, example = &2, example = &5, example = &10))]
pub struct Vout(u32);

impl Vout {
    pub const ZERO: Self = Vout(0);
    /// Also the protocol's coinbase vout.
    pub const MAX: Self = Vout(u32::MAX);

    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }
}

impl From<u16> for Vout {
    #[inline]
    fn from(value: u16) -> Self {
        Self(u32::from(value))
    }
}

impl From<u32> for Vout {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<usize> for Vout {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(
            u32::try_from(value).is_ok(),
            "vout {value} does not fit a u32"
        );
        Self(value as u32)
    }
}

impl From<Vout> for u32 {
    #[inline]
    fn from(value: Vout) -> Self {
        value.0
    }
}

impl From<Vout> for u64 {
    #[inline]
    fn from(value: Vout) -> Self {
        u64::from(value.0)
    }
}

impl From<Vout> for usize {
    #[inline]
    fn from(value: Vout) -> Self {
        value.0 as usize
    }
}

impl Display for Vout {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Vout {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

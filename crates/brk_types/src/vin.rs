use derive_more::Deref;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Input index in the spending transaction
#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(example = &0, example = &1, example = &2, example = &5, example = &10))]
pub struct Vin(u16);

impl Vin {
    const ZERO: Self = Vin(0);
    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }
}

impl From<u32> for Vin {
    #[inline]
    fn from(value: u32) -> Self {
        debug_assert!(value <= u16::MAX as u32, "vin {value} does not fit a u16");
        Self(value as u16)
    }
}

impl From<usize> for Vin {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(value <= u16::MAX as usize, "vin {value} does not fit a u16");
        Self(value as u16)
    }
}

impl From<Vin> for u64 {
    #[inline]
    fn from(value: Vin) -> Self {
        value.0 as u64
    }
}

impl From<Vin> for usize {
    #[inline]
    fn from(value: Vin) -> Self {
        value.0 as usize
    }
}

use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Input index in the spending transaction
#[derive(
    Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[schemars(example = &0, example = &1, example = &2, example = &5, example = &10)]
pub struct Vin(u16);

impl Vin {
    const ZERO: Self = Vin(0);
    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }
}

const U16_MAX_AS_U32: u32 = u16::MAX as u32;
impl From<u32> for Vin {
    #[inline]
    fn from(value: u32) -> Self {
        if value > U16_MAX_AS_U32 {
            panic!()
        }
        Self(value as u16)
    }
}

const U16_MAX_AS_USIZE: usize = u16::MAX as usize;
impl From<usize> for Vin {
    #[inline]
    fn from(value: usize) -> Self {
        if value > U16_MAX_AS_USIZE {
            panic!()
        }
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

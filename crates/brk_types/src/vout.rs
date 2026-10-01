#[cfg(feature = "storage")]
use itoa::Buffer;
use std::fmt::{Display, Formatter, Result};

use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// Index of the output being spent in the previous transaction
#[derive(
    Debug,
    Default,
    Deref,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
    Hash,
)]
#[cfg_attr(feature = "storage", derive(Bytes))]
#[schemars(example = &0, example = &1, example = &2, example = &5, example = &10)]
pub struct Vout(u16);

impl Vout {
    pub const ZERO: Self = Vout(0);
    pub const MAX: Self = Vout(u16::MAX);

    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    pub(crate) fn to_be_bytes(&self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
}

impl From<u16> for Vout {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}

const U16_MAX_AS_U32: u32 = u16::MAX as u32;
impl From<u32> for Vout {
    #[inline]
    fn from(value: u32) -> Self {
        if value > U16_MAX_AS_U32 {
            panic!()
        }
        Self(value as u16)
    }
}

const U16_MAX_AS_USIZE: usize = u16::MAX as usize;
impl From<usize> for Vout {
    #[inline]
    fn from(value: usize) -> Self {
        if value > U16_MAX_AS_USIZE {
            panic!()
        }
        Self(value as u16)
    }
}

impl From<Vout> for u16 {
    #[inline]
    fn from(value: Vout) -> Self {
        value.0
    }
}

impl From<Vout> for u32 {
    #[inline]
    fn from(value: Vout) -> Self {
        value.0 as u32
    }
}

impl From<Vout> for u64 {
    #[inline]
    fn from(value: Vout) -> Self {
        value.0 as u64
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

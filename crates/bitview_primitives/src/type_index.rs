use std::{
    fmt::{Display, Formatter, Result},
    ops::Add,
};

use brk_types::{CheckedSub, StoreValue};
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Index within its type (e.g., 0 for first P2WPKH address)
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    Hash,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct TypeIndex(u32);

impl TypeIndex {
    pub const COINBASE: Self = Self(u32::MAX);

    pub fn new(i: u32) -> Self {
        Self(i)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn copy_then_increment(&mut self) -> Self {
        let i = *self;
        self.increment();
        i
    }
}

impl From<u32> for TypeIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<TypeIndex> for u32 {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        value.0
    }
}

impl From<u64> for TypeIndex {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value as u32)
    }
}
impl From<TypeIndex> for u64 {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        value.0 as u64
    }
}

impl From<usize> for TypeIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}
impl From<TypeIndex> for usize {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        value.0 as usize
    }
}

impl Add<u32> for TypeIndex {
    type Output = Self;
    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl Add<usize> for TypeIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as u32)
    }
}

impl Add<TypeIndex> for TypeIndex {
    type Output = Self;
    fn add(self, rhs: TypeIndex) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl StoreValue for TypeIndex {
    type Bytes = [u8; 4];

    #[inline]
    fn to_store_bytes(&self) -> Self::Bytes {
        self.0.to_be_bytes()
    }

    #[inline]
    fn from_store_bytes(bytes: Self::Bytes) -> Self {
        Self(u32::from_be_bytes(bytes))
    }
}

impl CheckedSub<TypeIndex> for TypeIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<TypeIndex> for TypeIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Display for TypeIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for TypeIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

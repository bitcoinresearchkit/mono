use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    EmptyOutputIndex, OpReturnIndex, P2AAddrIndex, P2MSOutputIndex, P2PK33AddrIndex,
    P2PK65AddrIndex, P2PKHAddrIndex, P2SHAddrIndex, P2TRAddrIndex, P2WPKHAddrIndex, P2WSHAddrIndex,
    UnknownOutputIndex,
};
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

/// Fixed-size 32-bit unsigned integer optimized for on-disk storage
#[derive(
    Debug,
    Deref,
    Clone,
    Default,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct StoredU32(u32);

impl StoredU32 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn new(counter: u32) -> Self {
        Self(counter)
    }
}

impl From<u32> for StoredU32 {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<StoredU32> for u32 {
    #[inline]
    fn from(value: StoredU32) -> Self {
        value.0
    }
}

impl From<StoredU32> for f32 {
    #[inline]
    fn from(value: StoredU32) -> Self {
        value.0 as f32
    }
}

impl From<u64> for StoredU32 {
    #[inline]
    fn from(value: u64) -> Self {
        debug_assert!(value <= u32::MAX as u64);
        Self(value as u32)
    }
}

impl From<usize> for StoredU32 {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(value <= u32::MAX as usize);
        Self(value as u32)
    }
}

impl CheckedSub<StoredU32> for StoredU32 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<StoredU32> for StoredU32 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl CheckedSub<usize> for StoredU32 {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        debug_assert!(rhs <= u32::MAX as usize);
        self.0.checked_sub(rhs as u32).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<usize> for StoredU32 {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Div<usize> for StoredU32 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as u32)
    }
}

impl Add for StoredU32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for StoredU32 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for StoredU32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for StoredU32 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<usize> for StoredU32 {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        let res = self.0 as usize * rhs;
        debug_assert!(res <= u32::MAX as usize);
        Self::from(res)
    }
}

impl From<f64> for StoredU32 {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        debug_assert!(value <= u32::MAX as f64);
        Self(value as u32)
    }
}

impl From<StoredU32> for f64 {
    #[inline]
    fn from(value: StoredU32) -> Self {
        value.0 as f64
    }
}

impl From<StoredU32> for usize {
    #[inline]
    fn from(value: StoredU32) -> Self {
        value.0 as usize
    }
}

impl From<P2PK65AddrIndex> for StoredU32 {
    #[inline]
    fn from(value: P2PK65AddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2PK33AddrIndex> for StoredU32 {
    #[inline]
    fn from(value: P2PK33AddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2PKHAddrIndex> for StoredU32 {
    #[inline]
    fn from(value: P2PKHAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<OpReturnIndex> for StoredU32 {
    #[inline]
    fn from(value: OpReturnIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2MSOutputIndex> for StoredU32 {
    #[inline]
    fn from(value: P2MSOutputIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2SHAddrIndex> for StoredU32 {
    #[inline]
    fn from(value: P2SHAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2WSHAddrIndex> for StoredU32 {
    #[inline]
    fn from(value: P2WSHAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2WPKHAddrIndex> for StoredU32 {
    #[inline]
    fn from(value: P2WPKHAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2TRAddrIndex> for StoredU32 {
    #[inline]
    fn from(value: P2TRAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2AAddrIndex> for StoredU32 {
    #[inline]
    fn from(value: P2AAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<UnknownOutputIndex> for StoredU32 {
    #[inline]
    fn from(value: UnknownOutputIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<EmptyOutputIndex> for StoredU32 {
    #[inline]
    fn from(value: EmptyOutputIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl StoredU32 {
    fn index_name() -> &'static str {
        "u32"
    }
    fn index_aliases() -> &'static [&'static str] {
        &["u32"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for StoredU32 {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Display for StoredU32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for StoredU32 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

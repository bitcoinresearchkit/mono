use std::{
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Div, Sub, SubAssign},
};

use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    Day1, EmptyOutputIndex, Height, Month1, OpReturnIndex, P2AAddrIndex, P2MSOutputIndex,
    P2PK33AddrIndex, P2PK65AddrIndex, P2PKHAddrIndex, P2SHAddrIndex, P2TRAddrIndex,
    P2WPKHAddrIndex, P2WSHAddrIndex, StoredU32, TxInIndex, TxIndex, TxOutIndex, UnknownOutputIndex,
    Year1,
};
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

/// Fixed-size 64-bit unsigned integer optimized for on-disk storage
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
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct StoredU64(u64);

impl StoredU64 {
    pub const ZERO: Self = Self(0);

    pub fn new(counter: u64) -> Self {
        Self(counter)
    }
}

impl From<StoredU32> for StoredU64 {
    #[inline]
    fn from(value: StoredU32) -> Self {
        Self(u32::from(value) as u64)
    }
}

impl From<u64> for StoredU64 {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<StoredU64> for u64 {
    #[inline]
    fn from(value: StoredU64) -> Self {
        value.0
    }
}

impl From<StoredU64> for usize {
    #[inline]
    fn from(value: StoredU64) -> Self {
        value.0 as usize
    }
}

impl From<usize> for StoredU64 {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl CheckedSub<StoredU64> for StoredU64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<StoredU64> for StoredU64 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Div<usize> for StoredU64 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as u64)
    }
}

impl Add for StoredU64 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for StoredU64 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sum for StoredU64 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|v| v.0).sum())
    }
}

impl Sub for StoredU64 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for StoredU64 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl From<f64> for StoredU64 {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        Self(value as u64)
    }
}

impl From<StoredU64> for f64 {
    #[inline]
    fn from(value: StoredU64) -> Self {
        value.0 as f64
    }
}

impl From<TxIndex> for StoredU64 {
    #[inline]
    fn from(value: TxIndex) -> Self {
        Self(*value as u64)
    }
}

impl From<TxInIndex> for StoredU64 {
    #[inline]
    fn from(value: TxInIndex) -> Self {
        Self(*value)
    }
}

impl From<Height> for StoredU64 {
    #[inline]
    fn from(value: Height) -> Self {
        Self(*value as u64)
    }
}

impl From<TxOutIndex> for StoredU64 {
    #[inline]
    fn from(value: TxOutIndex) -> Self {
        Self(*value)
    }
}

impl From<Day1> for StoredU64 {
    #[inline]
    fn from(value: Day1) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<Month1> for StoredU64 {
    #[inline]
    fn from(value: Month1) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<Year1> for StoredU64 {
    #[inline]
    fn from(value: Year1) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<P2PK65AddrIndex> for StoredU64 {
    #[inline]
    fn from(value: P2PK65AddrIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<P2PK33AddrIndex> for StoredU64 {
    #[inline]
    fn from(value: P2PK33AddrIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<P2PKHAddrIndex> for StoredU64 {
    #[inline]
    fn from(value: P2PKHAddrIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<OpReturnIndex> for StoredU64 {
    #[inline]
    fn from(value: OpReturnIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<P2MSOutputIndex> for StoredU64 {
    #[inline]
    fn from(value: P2MSOutputIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<P2SHAddrIndex> for StoredU64 {
    #[inline]
    fn from(value: P2SHAddrIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<P2WSHAddrIndex> for StoredU64 {
    #[inline]
    fn from(value: P2WSHAddrIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<P2WPKHAddrIndex> for StoredU64 {
    #[inline]
    fn from(value: P2WPKHAddrIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<P2TRAddrIndex> for StoredU64 {
    #[inline]
    fn from(value: P2TRAddrIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<P2AAddrIndex> for StoredU64 {
    #[inline]
    fn from(value: P2AAddrIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<UnknownOutputIndex> for StoredU64 {
    #[inline]
    fn from(value: UnknownOutputIndex) -> Self {
        Self::from(u64::from(value))
    }
}

impl From<EmptyOutputIndex> for StoredU64 {
    #[inline]
    fn from(value: EmptyOutputIndex) -> Self {
        Self::from(u64::from(value))
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for StoredU64 {
    fn to_string() -> &'static str {
        "u64"
    }
    fn to_possible_strings() -> &'static [&'static str] {
        &["u64"]
    }
}

impl Display for StoredU64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for StoredU64 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

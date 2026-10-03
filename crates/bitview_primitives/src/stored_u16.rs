use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Div},
};

use brk_types::CheckedSub;
use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    EmptyOutputIndex, OpReturnIndex, P2AAddrIndex, P2MSOutputIndex, P2PK33AddrIndex,
    P2PK65AddrIndex, P2PKHAddrIndex, P2SHAddrIndex, P2TRAddrIndex, P2WPKHAddrIndex, P2WSHAddrIndex,
    UnknownOutputIndex,
};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

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
pub struct StoredU16(u16);

impl StoredU16 {
    pub fn new(v: u16) -> Self {
        Self(v)
    }
}

impl From<u16> for StoredU16 {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<usize> for StoredU16 {
    #[inline]
    fn from(value: usize) -> Self {
        debug_assert!(value <= u16::MAX as usize);
        Self(value as u16)
    }
}

impl CheckedSub<StoredU16> for StoredU16 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<StoredU16> for StoredU16 {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Div<usize> for StoredU16 {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as u16)
    }
}

impl Add for StoredU16 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for StoredU16 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl From<f64> for StoredU16 {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        debug_assert!(value <= u16::MAX as f64);
        Self(value as u16)
    }
}

impl From<StoredU16> for f64 {
    #[inline]
    fn from(value: StoredU16) -> Self {
        value.0 as f64
    }
}

impl From<StoredU16> for usize {
    #[inline]
    fn from(value: StoredU16) -> Self {
        value.0 as usize
    }
}

impl From<P2PK65AddrIndex> for StoredU16 {
    #[inline]
    fn from(value: P2PK65AddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2PK33AddrIndex> for StoredU16 {
    #[inline]
    fn from(value: P2PK33AddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2PKHAddrIndex> for StoredU16 {
    #[inline]
    fn from(value: P2PKHAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<OpReturnIndex> for StoredU16 {
    #[inline]
    fn from(value: OpReturnIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2MSOutputIndex> for StoredU16 {
    #[inline]
    fn from(value: P2MSOutputIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2SHAddrIndex> for StoredU16 {
    #[inline]
    fn from(value: P2SHAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2WSHAddrIndex> for StoredU16 {
    #[inline]
    fn from(value: P2WSHAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2WPKHAddrIndex> for StoredU16 {
    #[inline]
    fn from(value: P2WPKHAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2TRAddrIndex> for StoredU16 {
    #[inline]
    fn from(value: P2TRAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<P2AAddrIndex> for StoredU16 {
    #[inline]
    fn from(value: P2AAddrIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<UnknownOutputIndex> for StoredU16 {
    #[inline]
    fn from(value: UnknownOutputIndex) -> Self {
        Self::from(usize::from(value))
    }
}

impl From<EmptyOutputIndex> for StoredU16 {
    #[inline]
    fn from(value: EmptyOutputIndex) -> Self {
        Self::from(usize::from(value))
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for StoredU16 {
    fn to_string() -> &'static str {
        "u16"
    }
    fn to_possible_strings() -> &'static [&'static str] {
        &["u16"]
    }
}

impl Display for StoredU16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for StoredU16 {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

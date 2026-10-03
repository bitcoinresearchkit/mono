use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign},
};

use derive_more::{Deref, DerefMut};
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, StoreValue};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex, VecIndex};

/// Chain-wide transaction index (0 = the genesis coinbase). For an
/// in-block position, use `BlockTxIndex` instead.
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Deref,
    DerefMut,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    Hash,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
#[schemars(example = 0)]
pub struct TxIndex(u32);

impl TxIndex {
    pub const ZERO: Self = Self(0);
    pub fn new(tx_index: u32) -> Self {
        Self(tx_index)
    }

    pub fn incremented(self) -> Self {
        Self(*self + 1)
    }
}

impl Add<TxIndex> for TxIndex {
    type Output = Self;
    fn add(self, rhs: TxIndex) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<usize> for TxIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as u32)
    }
}

impl AddAssign<TxIndex> for TxIndex {
    fn add_assign(&mut self, rhs: TxIndex) {
        self.0 += rhs.0
    }
}

impl CheckedSub<TxIndex> for TxIndex {
    fn checked_sub(self, rhs: TxIndex) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(TxIndex::from)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<TxIndex> for TxIndex {
    fn checked_sub(self, rhs: TxIndex) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl From<u32> for TxIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<TxIndex> for u32 {
    #[inline]
    fn from(value: TxIndex) -> Self {
        value.0
    }
}

impl From<u64> for TxIndex {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value as u32)
    }
}
impl From<TxIndex> for u64 {
    #[inline]
    fn from(value: TxIndex) -> Self {
        value.0 as u64
    }
}

impl From<usize> for TxIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}
impl From<TxIndex> for usize {
    #[inline]
    fn from(value: TxIndex) -> Self {
        value.0 as usize
    }
}

impl StoreValue for TxIndex {
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

impl TxIndex {
    pub fn index_name() -> &'static str {
        "tx_index"
    }
    pub fn index_aliases() -> &'static [&'static str] {
        &["tx", "tx_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for TxIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for TxIndex {
    const INITIAL_CAPACITY: usize = 1_700_000_000;
}

impl Display for TxIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for TxIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

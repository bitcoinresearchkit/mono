use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign},
};

use brk_types::{CheckedSub, Vout};
use derive_more::{Deref, DerefMut};
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex, VecIndex};

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
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct TxOutIndex(u64);

impl TxOutIndex {
    pub const ZERO: Self = Self(0);

    pub const COINBASE: Self = Self(u64::MAX);

    pub fn new(index: u64) -> Self {
        Self(index)
    }

    pub fn is_coinbase(self) -> bool {
        self == Self::COINBASE
    }
}

impl Add<TxOutIndex> for TxOutIndex {
    type Output = Self;
    fn add(self, rhs: TxOutIndex) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<Vout> for TxOutIndex {
    type Output = Self;
    fn add(self, rhs: Vout) -> Self::Output {
        Self(self.0 + u64::from(rhs))
    }
}

impl Add<usize> for TxOutIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as u64)
    }
}

impl AddAssign<TxOutIndex> for TxOutIndex {
    fn add_assign(&mut self, rhs: TxOutIndex) {
        self.0 += rhs.0
    }
}

impl CheckedSub<TxOutIndex> for TxOutIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self::from)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<TxOutIndex> for TxOutIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl From<TxOutIndex> for u32 {
    #[inline]
    fn from(value: TxOutIndex) -> Self {
        if value.0 > u32::MAX as u64 {
            panic!()
        }
        value.0 as u32
    }
}

impl From<u64> for TxOutIndex {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl From<TxOutIndex> for u64 {
    #[inline]
    fn from(value: TxOutIndex) -> Self {
        value.0
    }
}

impl From<usize> for TxOutIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}
impl From<TxOutIndex> for usize {
    #[inline]
    fn from(value: TxOutIndex) -> Self {
        value.0 as usize
    }
}

impl TxOutIndex {
    pub(crate) fn index_name() -> &'static str {
        "txout_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["txo", "txout", "txout_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for TxOutIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for TxOutIndex {
    const INITIAL_CAPACITY: usize = 4_700_000_000;
}

impl Display for TxOutIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for TxOutIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

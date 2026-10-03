use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign},
};

use brk_types::{CheckedSub, Vin};
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
pub struct TxInIndex(u64);

impl TxInIndex {
    /// Sentinel value indicating an unspent output.
    pub const UNSPENT: Self = Self(u64::MAX);

    pub fn new(index: u64) -> Self {
        Self(index)
    }

    pub fn is_unspent(self) -> bool {
        self == Self::UNSPENT
    }
}

impl Add<TxInIndex> for TxInIndex {
    type Output = Self;
    fn add(self, rhs: TxInIndex) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<Vin> for TxInIndex {
    type Output = Self;
    fn add(self, rhs: Vin) -> Self::Output {
        Self(self.0 + u64::from(rhs))
    }
}

impl Add<usize> for TxInIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as u64)
    }
}

impl AddAssign<TxInIndex> for TxInIndex {
    fn add_assign(&mut self, rhs: TxInIndex) {
        self.0 += rhs.0
    }
}

impl CheckedSub<TxInIndex> for TxInIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self::from)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<TxInIndex> for TxInIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl From<TxInIndex> for u32 {
    #[inline]
    fn from(value: TxInIndex) -> Self {
        if value.0 > u32::MAX as u64 {
            panic!()
        }
        value.0 as u32
    }
}

impl From<u64> for TxInIndex {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl From<TxInIndex> for u64 {
    #[inline]
    fn from(value: TxInIndex) -> Self {
        value.0
    }
}

impl From<usize> for TxInIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}
impl From<TxInIndex> for usize {
    #[inline]
    fn from(value: TxInIndex) -> Self {
        value.0 as usize
    }
}

impl TxInIndex {
    pub(crate) fn index_name() -> &'static str {
        "txin_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["txi", "txin", "txin_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for TxInIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for TxInIndex {
    const INITIAL_CAPACITY: usize = 4_200_000_000;
}

impl Display for TxInIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for TxInIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

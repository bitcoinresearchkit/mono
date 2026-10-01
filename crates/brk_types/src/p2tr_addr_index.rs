use std::{
    fmt::{Display, Formatter, Result},
    ops::Add,
};

use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, TypeIndex};

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
pub struct P2TRAddrIndex(TypeIndex);

impl From<TypeIndex> for P2TRAddrIndex {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}

impl From<P2TRAddrIndex> for TypeIndex {
    #[inline]
    fn from(value: P2TRAddrIndex) -> Self {
        value.0
    }
}

impl From<P2TRAddrIndex> for u32 {
    #[inline]
    fn from(value: P2TRAddrIndex) -> Self {
        Self::from(*value)
    }
}

impl From<P2TRAddrIndex> for u64 {
    #[inline]
    fn from(value: P2TRAddrIndex) -> Self {
        Self::from(*value)
    }
}

impl From<u32> for P2TRAddrIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self(TypeIndex::from(value))
    }
}

impl From<P2TRAddrIndex> for usize {
    #[inline]
    fn from(value: P2TRAddrIndex) -> Self {
        Self::from(*value)
    }
}

impl From<usize> for P2TRAddrIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}

impl Add<usize> for P2TRAddrIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(*self + rhs)
    }
}

impl CheckedSub<P2TRAddrIndex> for P2TRAddrIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<P2TRAddrIndex> for P2TRAddrIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl P2TRAddrIndex {
    pub(crate) fn index_name() -> &'static str {
        "p2tr_addr_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["traddr", "p2traddr", "p2tr_addr_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for P2TRAddrIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for P2TRAddrIndex {
    const INITIAL_CAPACITY: usize = 100_000_000;
}

impl Display for P2TRAddrIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for P2TRAddrIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

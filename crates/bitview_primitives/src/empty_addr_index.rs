use std::{
    fmt::{Display, Formatter, Result},
    ops::Add,
};

use brk_types::CheckedSub;
use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::TypeIndex;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex, VecIndex};

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deref,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct EmptyAddrIndex(TypeIndex);

impl From<TypeIndex> for EmptyAddrIndex {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}

impl From<usize> for EmptyAddrIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}
impl From<u32> for EmptyAddrIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self(TypeIndex::from(value))
    }
}

impl From<EmptyAddrIndex> for usize {
    #[inline]
    fn from(value: EmptyAddrIndex) -> Self {
        usize::from(value.0)
    }
}

impl Add<usize> for EmptyAddrIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl CheckedSub<EmptyAddrIndex> for EmptyAddrIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<EmptyAddrIndex> for EmptyAddrIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl EmptyAddrIndex {
    pub(crate) fn index_name() -> &'static str {
        "empty_addr_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["emptyaddr", "empty_addr_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for EmptyAddrIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for EmptyAddrIndex {
    const INITIAL_CAPACITY: usize = 1_800_000_000;
}

impl Display for EmptyAddrIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for EmptyAddrIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

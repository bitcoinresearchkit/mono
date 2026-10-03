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
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Deref,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct FundedAddrIndex(TypeIndex);

impl From<TypeIndex> for FundedAddrIndex {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}

impl From<usize> for FundedAddrIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}
impl From<FundedAddrIndex> for usize {
    #[inline]
    fn from(value: FundedAddrIndex) -> Self {
        usize::from(value.0)
    }
}
impl From<FundedAddrIndex> for u32 {
    #[inline]
    fn from(value: FundedAddrIndex) -> Self {
        u32::from(value.0)
    }
}
impl Add<usize> for FundedAddrIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl CheckedSub<FundedAddrIndex> for FundedAddrIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<FundedAddrIndex> for FundedAddrIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}
impl FundedAddrIndex {
    pub(crate) fn index_name() -> &'static str {
        "funded_addr_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["fundedaddr", "funded_addr_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for FundedAddrIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for FundedAddrIndex {
    const INITIAL_CAPACITY: usize = 70_000_000;
}

impl Display for FundedAddrIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for FundedAddrIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

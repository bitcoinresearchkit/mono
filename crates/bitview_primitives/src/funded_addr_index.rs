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
pub struct FundedAddressIndex(TypeIndex);

impl From<TypeIndex> for FundedAddressIndex {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}

impl From<usize> for FundedAddressIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}
impl From<FundedAddressIndex> for usize {
    #[inline]
    fn from(value: FundedAddressIndex) -> Self {
        usize::from(value.0)
    }
}
impl From<FundedAddressIndex> for u32 {
    #[inline]
    fn from(value: FundedAddressIndex) -> Self {
        u32::from(value.0)
    }
}
impl Add<usize> for FundedAddressIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl CheckedSub<FundedAddressIndex> for FundedAddressIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<FundedAddressIndex> for FundedAddressIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}
impl FundedAddressIndex {
    pub(crate) fn index_name() -> &'static str {
        "funded_address_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["fundedaddr", "funded_address_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for FundedAddressIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for FundedAddressIndex {
    const INITIAL_CAPACITY: usize = 70_000_000;
}

impl Display for FundedAddressIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for FundedAddressIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

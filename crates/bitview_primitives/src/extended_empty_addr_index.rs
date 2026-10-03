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

/// Index into the sidecar for empty-address data that does not fit inline.
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
pub struct ExtendedEmptyAddrIndex(TypeIndex);

impl From<TypeIndex> for ExtendedEmptyAddrIndex {
    #[inline(always)]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}

impl From<usize> for ExtendedEmptyAddrIndex {
    #[inline(always)]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}

impl From<ExtendedEmptyAddrIndex> for usize {
    #[inline(always)]
    fn from(value: ExtendedEmptyAddrIndex) -> Self {
        usize::from(value.0)
    }
}

impl From<ExtendedEmptyAddrIndex> for u32 {
    #[inline(always)]
    fn from(value: ExtendedEmptyAddrIndex) -> Self {
        u32::from(value.0)
    }
}

impl Add<usize> for ExtendedEmptyAddrIndex {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl CheckedSub for ExtendedEmptyAddrIndex {
    #[inline(always)]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for ExtendedEmptyAddrIndex {
    #[inline(always)]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl ExtendedEmptyAddrIndex {
    pub(crate) fn index_name() -> &'static str {
        "extended_empty_addr_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["extendedemptyaddr", "extended_empty_addr_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for ExtendedEmptyAddrIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for ExtendedEmptyAddrIndex {
    const INITIAL_CAPACITY: usize = 1 << 30;
}

impl Display for ExtendedEmptyAddrIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for ExtendedEmptyAddrIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

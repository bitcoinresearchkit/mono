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
pub struct ExtendedEmptyAddressIndex(TypeIndex);

impl From<TypeIndex> for ExtendedEmptyAddressIndex {
    #[inline(always)]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}

impl From<usize> for ExtendedEmptyAddressIndex {
    #[inline(always)]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}

impl From<ExtendedEmptyAddressIndex> for usize {
    #[inline(always)]
    fn from(value: ExtendedEmptyAddressIndex) -> Self {
        usize::from(value.0)
    }
}

impl From<ExtendedEmptyAddressIndex> for u32 {
    #[inline(always)]
    fn from(value: ExtendedEmptyAddressIndex) -> Self {
        u32::from(value.0)
    }
}

impl Add<usize> for ExtendedEmptyAddressIndex {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl CheckedSub for ExtendedEmptyAddressIndex {
    #[inline(always)]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for ExtendedEmptyAddressIndex {
    #[inline(always)]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl ExtendedEmptyAddressIndex {
    pub(crate) fn index_name() -> &'static str {
        "extended_empty_address_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["extendedemptyaddr", "extended_empty_address_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for ExtendedEmptyAddressIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for ExtendedEmptyAddressIndex {
    const INITIAL_CAPACITY: usize = 1 << 30;
}

impl Display for ExtendedEmptyAddressIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for ExtendedEmptyAddressIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

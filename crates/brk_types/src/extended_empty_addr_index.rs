use std::ops::Add;

use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{CheckedSub, Formattable, Pco, PrintableIndex, VecIndex};

use crate::TypeIndex;

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
    Pco,
    JsonSchema,
)]
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
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl PrintableIndex for ExtendedEmptyAddrIndex {
    fn to_string() -> &'static str {
        "extended_empty_addr_index"
    }

    fn to_possible_strings() -> &'static [&'static str] {
        &["extendedemptyaddr", "extended_empty_addr_index"]
    }
}

impl VecIndex for ExtendedEmptyAddrIndex {
    const INITIAL_CAPACITY: usize = 1 << 30;
}

impl std::fmt::Display for ExtendedEmptyAddrIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Formattable for ExtendedEmptyAddrIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

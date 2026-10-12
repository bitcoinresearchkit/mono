use std::{
    fmt::{Display, Formatter, Result},
    ops::Add,
};

use brk_types::CheckedSub;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::TypeIndex;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex, VecIndex};

/// Index of a P2PK address with a compressed (33-byte) public key.
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
pub struct P2PK33AddressIndex(TypeIndex);

impl From<TypeIndex> for P2PK33AddressIndex {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}

impl From<P2PK33AddressIndex> for TypeIndex {
    #[inline]
    fn from(value: P2PK33AddressIndex) -> Self {
        value.0
    }
}

impl From<P2PK33AddressIndex> for u32 {
    #[inline]
    fn from(value: P2PK33AddressIndex) -> Self {
        Self::from(*value)
    }
}

impl From<P2PK33AddressIndex> for u64 {
    #[inline]
    fn from(value: P2PK33AddressIndex) -> Self {
        Self::from(*value)
    }
}

impl From<u32> for P2PK33AddressIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self(TypeIndex::from(value))
    }
}

impl From<P2PK33AddressIndex> for usize {
    #[inline]
    fn from(value: P2PK33AddressIndex) -> Self {
        Self::from(*value)
    }
}

impl From<usize> for P2PK33AddressIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}

impl Add<usize> for P2PK33AddressIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(*self + rhs)
    }
}

impl CheckedSub<P2PK33AddressIndex> for P2PK33AddressIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<P2PK33AddressIndex> for P2PK33AddressIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl P2PK33AddressIndex {
    pub(crate) fn index_name() -> &'static str {
        "p2pk33_address_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["pk33addr", "p2pk33addr", "p2pk33_address_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for P2PK33AddressIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for P2PK33AddressIndex {
    const INITIAL_CAPACITY: usize = 40_000;
}

impl Display for P2PK33AddressIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for P2PK33AddressIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

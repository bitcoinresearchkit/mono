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

/// Index of a P2PKH address.
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
pub struct P2PKHAddressIndex(TypeIndex);

impl From<TypeIndex> for P2PKHAddressIndex {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}

impl From<P2PKHAddressIndex> for TypeIndex {
    #[inline]
    fn from(value: P2PKHAddressIndex) -> Self {
        value.0
    }
}

impl From<P2PKHAddressIndex> for usize {
    #[inline]
    fn from(value: P2PKHAddressIndex) -> Self {
        Self::from(*value)
    }
}

impl From<P2PKHAddressIndex> for u64 {
    #[inline]
    fn from(value: P2PKHAddressIndex) -> Self {
        Self::from(*value)
    }
}

impl From<P2PKHAddressIndex> for u32 {
    #[inline]
    fn from(value: P2PKHAddressIndex) -> Self {
        Self::from(*value)
    }
}

impl From<u32> for P2PKHAddressIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self(TypeIndex::from(value))
    }
}

impl From<usize> for P2PKHAddressIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}

impl Add<usize> for P2PKHAddressIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(*self + rhs)
    }
}

impl CheckedSub<P2PKHAddressIndex> for P2PKHAddressIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<P2PKHAddressIndex> for P2PKHAddressIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl P2PKHAddressIndex {
    pub(crate) fn index_name() -> &'static str {
        "p2pkh_address_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["pkhaddr", "p2pkhaddr", "p2pkh_address_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for P2PKHAddressIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for P2PKHAddressIndex {
    const INITIAL_CAPACITY: usize = 750_000_000;
}

impl Display for P2PKHAddressIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for P2PKHAddressIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

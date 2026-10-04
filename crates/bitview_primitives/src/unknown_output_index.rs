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

/// Index of an output with an unrecognized script.
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
pub struct UnknownOutputIndex(TypeIndex);

impl From<TypeIndex> for UnknownOutputIndex {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}
impl From<UnknownOutputIndex> for u64 {
    #[inline]
    fn from(value: UnknownOutputIndex) -> Self {
        Self::from(*value)
    }
}
impl From<UnknownOutputIndex> for usize {
    #[inline]
    fn from(value: UnknownOutputIndex) -> Self {
        Self::from(*value)
    }
}
impl From<usize> for UnknownOutputIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}
impl Add<usize> for UnknownOutputIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(*self + rhs)
    }
}
impl CheckedSub<UnknownOutputIndex> for UnknownOutputIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<UnknownOutputIndex> for UnknownOutputIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl UnknownOutputIndex {
    pub(crate) fn index_name() -> &'static str {
        "unknown_output_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["unknownout", "unknown_output_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for UnknownOutputIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for UnknownOutputIndex {
    const INITIAL_CAPACITY: usize = 200_000;
}

impl Display for UnknownOutputIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for UnknownOutputIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

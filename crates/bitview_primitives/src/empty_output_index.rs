use std::{fmt, ops::Add};

use brk_types::CheckedSub;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::TypeIndex;

#[cfg(feature = "storage")]
use vecdb::{CheckedSub as VecdbCheckedSub, Formattable, Pco, PrintableIndex};

/// Index of an output with an empty script.
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
pub struct EmptyOutputIndex(TypeIndex);
impl From<TypeIndex> for EmptyOutputIndex {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}
impl From<EmptyOutputIndex> for u64 {
    #[inline]
    fn from(value: EmptyOutputIndex) -> Self {
        Self::from(value.0)
    }
}
impl From<EmptyOutputIndex> for usize {
    #[inline]
    fn from(value: EmptyOutputIndex) -> Self {
        Self::from(value.0)
    }
}
impl From<usize> for EmptyOutputIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}
impl Add<usize> for EmptyOutputIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl CheckedSub<EmptyOutputIndex> for EmptyOutputIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<EmptyOutputIndex> for EmptyOutputIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl EmptyOutputIndex {
    pub(crate) fn index_name() -> &'static str {
        "empty_output_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["emptyout", "empty_output_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for EmptyOutputIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl fmt::Display for EmptyOutputIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for EmptyOutputIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

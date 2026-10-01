use std::{
    fmt::{Display, Formatter, Result},
    ops::Add,
};

use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, TypeIndex};

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

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
pub struct OpReturnIndex(TypeIndex);

impl From<TypeIndex> for OpReturnIndex {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        Self(value)
    }
}

impl From<OpReturnIndex> for usize {
    #[inline]
    fn from(value: OpReturnIndex) -> Self {
        Self::from(*value)
    }
}

impl From<OpReturnIndex> for u64 {
    #[inline]
    fn from(value: OpReturnIndex) -> Self {
        Self::from(*value)
    }
}

impl From<usize> for OpReturnIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(TypeIndex::from(value))
    }
}

impl Add<usize> for OpReturnIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(*self + rhs)
    }
}

impl CheckedSub<OpReturnIndex> for OpReturnIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self.0, rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<OpReturnIndex> for OpReturnIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl OpReturnIndex {
    pub(crate) fn index_name() -> &'static str {
        "op_return_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["op", "opreturn", "op_return", "op_return_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for OpReturnIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Display for OpReturnIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "storage")]
impl Formattable for OpReturnIndex {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf);
    }
}

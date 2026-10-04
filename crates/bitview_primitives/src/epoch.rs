use std::{
    fmt::{Debug, Display, Formatter, Result},
    ops::{Add, AddAssign},
};

use brk_types::{BLOCKS_PER_DIFF_EPOCHS, CheckedSub, Height};
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FromCoarserIndex;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex};

/// Index of a difficulty epoch (2,016 blocks).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Epoch(u16);

impl From<u16> for Epoch {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<usize> for Epoch {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u16)
    }
}

impl From<Epoch> for usize {
    #[inline]
    fn from(value: Epoch) -> Self {
        value.0 as usize
    }
}

impl Add for Epoch {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.0 + rhs.0)
    }
}

impl AddAssign for Epoch {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Add<usize> for Epoch {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::from(self.0 + rhs as u16)
    }
}

impl From<Height> for Epoch {
    #[inline]
    fn from(value: Height) -> Self {
        Self((u32::from(value) / BLOCKS_PER_DIFF_EPOCHS) as u16)
    }
}

impl CheckedSub for Epoch {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for Epoch {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl Epoch {
    pub(crate) fn index_name() -> &'static str {
        "epoch"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["epoch", "difficulty", "difficultyepoch", "diff"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Epoch {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

impl Display for Epoch {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for Epoch {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

impl From<f64> for Epoch {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        Self(value.round() as u16)
    }
}

impl From<Epoch> for f64 {
    #[inline]
    fn from(value: Epoch) -> Self {
        value.0 as f64
    }
}

impl FromCoarserIndex<Epoch> for Height {
    fn max_from_(coarser: Epoch) -> usize {
        (usize::from(coarser) + 1) * BLOCKS_PER_DIFF_EPOCHS as usize - 1
    }
}

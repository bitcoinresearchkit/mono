use brk_types::{BLOCKS_PER_DIFF_EPOCH, Height};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FromCoarserIndex;

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Index of a difficulty epoch (2,016 blocks).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct DifficultyEpoch(u16);

brk_types::impl_integer_index!(DifficultyEpoch, u16);

impl From<Height> for DifficultyEpoch {
    #[inline]
    fn from(value: Height) -> Self {
        Self((u32::from(value) / BLOCKS_PER_DIFF_EPOCH) as u16)
    }
}

impl DifficultyEpoch {
    pub(crate) fn index_name() -> &'static str {
        "difficulty_epoch"
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for DifficultyEpoch {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}

impl From<f64> for DifficultyEpoch {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value.round() as u16)
    }
}

impl From<DifficultyEpoch> for f64 {
    #[inline]
    fn from(value: DifficultyEpoch) -> Self {
        value.0 as f64
    }
}

impl FromCoarserIndex<DifficultyEpoch> for Height {
    fn max_from_unbounded(coarser: DifficultyEpoch) -> usize {
        (usize::from(coarser) + 1) * BLOCKS_PER_DIFF_EPOCH as usize - 1
    }
}

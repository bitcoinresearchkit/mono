use brk_types::{BLOCKS_PER_DIFF_EPOCHS, Height};
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
pub struct Epoch(u16);

brk_types::impl_integer_index!(Epoch, u16);

impl From<Height> for Epoch {
    #[inline]
    fn from(value: Height) -> Self {
        Self((u32::from(value) / BLOCKS_PER_DIFF_EPOCHS) as u16)
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

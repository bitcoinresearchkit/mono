use brk_types::Height;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FromCoarserIndex;

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Index of a halving epoch (210,000 blocks).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct HalvingEpoch(u32);

brk_types::impl_integer_index!(HalvingEpoch, u32);

impl From<Height> for HalvingEpoch {
    #[inline]
    fn from(value: Height) -> Self {
        Self(u32::from(value) / brk_types::chain().halving_interval)
    }
}

impl HalvingEpoch {
    pub(crate) fn index_name() -> &'static str {
        "halving_epoch"
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for HalvingEpoch {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}

impl From<f64> for HalvingEpoch {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value.round() as u32)
    }
}

impl From<HalvingEpoch> for f64 {
    #[inline]
    fn from(value: HalvingEpoch) -> Self {
        value.0 as f64
    }
}

impl FromCoarserIndex<HalvingEpoch> for Height {
    fn max_from_unbounded(coarser: HalvingEpoch) -> usize {
        (usize::from(coarser) + 1) * brk_types::chain().halving_interval as usize - 1
    }
}

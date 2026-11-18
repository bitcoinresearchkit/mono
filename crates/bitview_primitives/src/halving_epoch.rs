use brk_types::Height;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FromCoarserIndex;

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex};

/// Index of a subsidy halving epoch on the active chain.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct HalvingEpoch(u8);

brk_types::impl_integer_index!(HalvingEpoch, u8);

impl From<Height> for HalvingEpoch {
    #[inline]
    fn from(value: Height) -> Self {
        let epoch = u32::from(value) / brk_types::chain().halving_interval;
        Self(u8::try_from(epoch).expect("the indexer stops at HalvingEpoch::last_height"))
    }
}

impl HalvingEpoch {
    /// The last height whose epoch the `u8` holds: 38,399 on regtest, past 53 million on the
    /// others. The indexer refuses a chain past it.
    pub fn last_height() -> Height {
        Height::new((u32::from(u8::MAX) + 1) * brk_types::chain().halving_interval - 1)
    }

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
        Self(value.round() as u8)
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

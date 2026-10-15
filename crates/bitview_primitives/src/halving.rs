use brk_types::{BLOCKS_PER_HALVING, Height};
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
pub struct Halving(u8);

brk_types::impl_integer_index!(Halving, u8);

impl Halving {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}

impl From<Height> for Halving {
    #[inline]
    fn from(value: Height) -> Self {
        Self((u32::from(value) / BLOCKS_PER_HALVING) as u8)
    }
}

impl Halving {
    pub(crate) fn index_name() -> &'static str {
        "halving"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["halving", "halvingepoch", "halv"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for Halving {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}

impl From<f64> for Halving {
    #[inline]
    fn from(value: f64) -> Self {
        let value = value.max(0.0);
        Self(value.round() as u8)
    }
}

impl From<Halving> for f64 {
    #[inline]
    fn from(value: Halving) -> Self {
        value.0 as f64
    }
}

impl FromCoarserIndex<Halving> for Height {
    fn max_from_(coarser: Halving) -> usize {
        (usize::from(coarser) + 1) * BLOCKS_PER_HALVING as usize - 1
    }
}

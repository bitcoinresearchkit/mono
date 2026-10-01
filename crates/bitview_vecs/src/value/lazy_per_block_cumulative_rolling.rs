use bitview_collections::Windows;
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::ReadableCloneableVec;

use crate::{IndexSources, LazyCumulativeValuePerBlock, LazyValueBlock, RollingAmountTotals};

#[derive(Clone, derive_more::Deref, derive_more::DerefMut, Traversable)]
pub struct LazyValuePerBlockCumulativeRolling {
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyValueBlock,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyCumulativeValuePerBlock,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    rolling: RollingAmountTotals,
}

impl LazyValuePerBlockCumulativeRolling {
    pub fn from_cumulative_sources(
        name: &str,
        version: Version,
        cumulative_sats: &(impl ReadableCloneableVec<Height, Sats> + ?Sized),
        cumulative_cents: &(impl ReadableCloneableVec<Height, Cents> + ?Sized),
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Self {
        let cumulative = LazyCumulativeValuePerBlock::from_sources(
            &format!("{name}_cumulative"),
            version,
            cumulative_sats,
            cumulative_cents,
            indexes,
        );
        let block = LazyValueBlock::from_cumulative_sources(
            name,
            version,
            &cumulative.sats.height,
            &cumulative.cents.height,
        );
        let rolling = RollingAmountTotals::new(
            name,
            version,
            &cumulative.sats.height,
            &cumulative.cents.height,
            window_starts,
            indexes,
        );

        Self {
            block,
            cumulative,
            rolling,
        }
    }
}

use brk_traversable::Traversable;
use brk_types::{Height, PartsPerMillion32, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{CachedBoxedVec, ColumnId};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, LazyColumnPerBlockCumulativeRolling, LazyPercentCumulativeRolling,
        RatioSats, Windows,
    },
};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct FeesSeries<C: ColumnId> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub fees: LazyColumnPerBlockCumulativeRolling<Sats, C>,
    /// Fees of transactions in the selected bucket divided by all transaction
    /// fees over the same cumulative or trailing window.
    pub fee_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

impl<C: ColumnId> FeesSeries<C> {
    pub(super) fn new(
        prefix: &str,
        version: Version,
        fees: LazyColumnPerBlockCumulativeRolling<Sats, C>,
        chain_fees: CachedBoxedVec<Height, Sats>,
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let fee_share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
            Sats,
            Sats,
            RatioSats<PartsPerMillion32>,
        >(
            &format!("{prefix}_fee_share"),
            version,
            &fees.cumulative.height,
            chain_fees,
            cached_starts,
            indexes,
        );

        Self { fees, fee_share }
    }
}

use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::ReadableBoxedVec;

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, CentsUnsignedToDollars, Identity, LazyCumulativeValuePerBlock,
        LazyPerBlock, LazyRollingAvgsAmountFromHeight, LazyRollingSumsAmountFromHeight,
        LazyValueBlock, SatsToBitcoin, Windows,
    },
};

#[derive(Clone, Traversable)]
pub struct LazyValuePerBlockCumulativeRolling {
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyValueBlock,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyCumulativeValuePerBlock,
    pub sum: LazyRollingSumsAmountFromHeight,
    pub average: LazyRollingAvgsAmountFromHeight,
}

impl LazyValuePerBlockCumulativeRolling {
    pub(crate) fn from_boxed_cumulative_sources(
        name: &str,
        version: Version,
        cumulative_sats: ReadableBoxedVec<Height, Sats>,
        cumulative_cents: ReadableBoxedVec<Height, Cents>,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let cumulative_name = format!("{name}_cumulative");
        let sats = LazyPerBlock::from_boxed_height_source::<Identity<Sats>>(
            &format!("{cumulative_name}_sats"),
            version,
            cumulative_sats,
            indexes,
        );
        let btc = LazyPerBlock::from_lazy::<SatsToBitcoin, Sats>(&cumulative_name, version, &sats);
        let cents = LazyPerBlock::from_boxed_height_source::<Identity<Cents>>(
            &format!("{cumulative_name}_cents"),
            version,
            cumulative_cents,
            indexes,
        );
        let usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(
            &format!("{cumulative_name}_usd"),
            version,
            &cents,
        );
        let cumulative = LazyCumulativeValuePerBlock {
            btc,
            sats,
            usd,
            cents,
        };
        let block = LazyValueBlock::from_cumulative_sources(
            name,
            version,
            &cumulative.sats.height,
            &cumulative.cents.height,
        );
        let sum = LazyRollingSumsAmountFromHeight::new(
            &format!("{name}_sum"),
            version,
            &cumulative.sats.height,
            &cumulative.cents.height,
            cached_starts,
            indexes,
        );
        let average = LazyRollingAvgsAmountFromHeight::new(
            &format!("{name}_average"),
            version,
            &cumulative.sats.height,
            &cumulative.cents.height,
            cached_starts,
            indexes,
        );

        Self {
            block,
            cumulative,
            sum,
            average,
        }
    }
}

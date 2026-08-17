use brk_error::Result;
use brk_indexer::Indexer;
use brk_traversable::Traversable;
use brk_types::{PartsPerMillion32, PoolSlug};
use derive_more::{Deref, DerefMut};
use vecdb::{BinaryTransform, Database, Exit, Rw, StorageMode, Version};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, LazyPercentRollingWindows, MaskSats, ValuePerBlockCumulativeRolling,
        Windows,
    },
    mining, price,
};

use super::minor;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub base: minor::Vecs,

    /// Coinbase transaction output value for blocks attributed to the selected
    /// pool, and zero for other blocks. USD and cents representations value
    /// each included reward at that block's spot price.
    pub rewards: ValuePerBlockCumulativeRolling<M>,
    /// Share of blocks in the named trailing timestamp window attributed to the
    /// selected pool: pool block count divided by total chain block count in
    /// that window.
    #[traversable(rename = "dominance")]
    pub dominance_rolling: LazyPercentRollingWindows<PartsPerMillion32>,
}

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        slug: PoolSlug,
        pool_heights: super::PoolHeights,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let suffix = |s: &str| format!("{}_{s}", slug);

        let base = minor::Vecs::forced_import(slug, pool_heights, version, indexes, cached_starts);

        let rewards = ValuePerBlockCumulativeRolling::forced_import(
            db,
            &suffix("rewards"),
            version,
            indexes,
            cached_starts,
        )?;

        let dominance_rolling = LazyPercentRollingWindows::from_compact_cumulative_average(
            &suffix("dominance"),
            version,
            &base.blocks_mined.cumulative.height,
            cached_starts,
            indexes,
        );

        Ok(Self {
            base,
            rewards,
            dominance_rolling,
        })
    }

    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &price::Vecs,
        mining: &mining::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.rewards.compute_from_pair(
            starting_height,
            prices,
            &self.base.blocks_mined.block,
            &mining.rewards.coinbase.block.sats,
            |_, mask, value| MaskSats::apply(mask, value),
            exit,
        )?;
        Ok(())
    }
}

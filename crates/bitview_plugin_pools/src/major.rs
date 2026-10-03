use bitview_collections::Windows;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_mining::Vecs as MiningVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_transforms::MaskSats;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPercentRollingWindows, LazyWindowStartVec, ValuePerBlockCumulativeRolling};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{PartsPerMillion32, PoolSlug};
use derive_more::{Deref, DerefMut};
use vecdb::{BinaryTransform, Database, Rw, StorageMode, Version};

use super::{PoolHeights, minor};

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub base: minor::Vecs,

    /// Coinbase transaction output value for blocks attributed to a mining
    /// pool, and zero for other blocks. USD and cents representations value
    /// each included reward at that block's spot price.
    pub rewards: ValuePerBlockCumulativeRolling<M>,
    /// Share of blocks in a trailing timestamp window attributed to a mining
    /// pool: pool block count divided by total chain block count in that
    /// window.
    #[traversable(rename = "dominance")]
    pub dominance_rolling: LazyPercentRollingWindows<PartsPerMillion32>,
}

impl Vecs {
    pub fn import(
        db: &Database,
        slug: PoolSlug,
        pool_heights: PoolHeights,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let suffix = |s: &str| format!("{}_{s}", slug);

        let base = minor::Vecs::new(slug, pool_heights, version, mappings, window_starts);

        let rewards = ValuePerBlockCumulativeRolling::import(
            db,
            &suffix("rewards"),
            version,
            mappings,
            window_starts,
        )?;

        let dominance_rolling = LazyPercentRollingWindows::from_cumulative_average(
            &suffix("dominance"),
            version,
            &base.blocks_mined.cumulative.height,
            window_starts,
            mappings,
        );

        Ok(Self {
            base,
            rewards,
            dominance_rolling,
        })
    }

    pub fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &PriceVecs,
        mining: &MiningVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.rewards.compute_from_pair(
            starting_height,
            &prices.spot.cents.height,
            &self.base.blocks_mined.block,
            &mining.rewards.coinbase.block.sats,
            |_, mask, value| MaskSats::apply(mask, value),
            exit,
        )?;
        Ok(())
    }
}

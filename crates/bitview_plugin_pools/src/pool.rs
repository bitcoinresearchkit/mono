use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, Float64, Hashrate, PartsPerMillion32, PoolSlug};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyCumulativeValuePerBlock, LazyIndexedVec, LazyPerBlock, LazyPercentPerBlock,
    LazyPercentRollingWindows, LazyRollingSumsAmountFromHeight, LazyRollingSumsFromHeight,
    LazyWindowStartVec, ops::Quotient, shapes::Windows,
};
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats};
use vecdb::{
    BinaryTransform, DeltaSub, Ident, LazyDeltaVec, LazyVec, ReadableCloneableVec, Version,
};

use crate::pool_heights::{Column, PoolCumulativeVec, PoolHeights};

#[derive(Clone, Traversable)]
pub struct PoolVecs {
    /// Blocks attributed to the pool.
    pub block_count: BlockCount,
    /// Share of all blocks attributed to the pool: from genesis through the represented block
    /// (`cumulative`) or over a trailing window.
    block_share: BlockShare,
    /// Coinbase output value of the pool's blocks, with USD valuing each block's reward at its
    /// spot price.
    pub rewards: Rewards,
    /// Transaction fees per block the pool mined over a trailing window: the fees of its blocks
    /// in the window divided by their number; null when it mined none.
    fees_per_block: Windows<FeesPerBlock>,
    /// Estimated hash rate over a trailing window, in hashes per second: the pool's share of the
    /// window's blocks times the network hash-rate estimate for the same window.
    hashrate: Windows<LazyPerBlock<Hashrate>>,
}

#[derive(Clone, Traversable)]
pub struct BlockCount {
    /// Through the represented block, from genesis.
    pub cumulative: LazyPerBlock<Count>,
    pub sum: LazyRollingSumsFromHeight<Count>,
}

#[derive(Clone, Traversable)]
pub struct BlockShare {
    pub cumulative: LazyPercentPerBlock<PartsPerMillion32>,
    #[traversable(flatten)]
    pub windows: LazyPercentRollingWindows<PartsPerMillion32>,
}

#[derive(Clone, Traversable)]
pub struct Rewards {
    /// Through the represented block, from genesis.
    pub cumulative: LazyCumulativeValuePerBlock,
    pub sum: LazyRollingSumsAmountFromHeight,
}

#[derive(Clone, Traversable)]
pub struct FeesPerBlock {
    /// In BTC.
    pub btc: LazyPerBlock<Bitcoin>,
    /// In US dollars, each block's fees valued at its spot price.
    pub usd: LazyPerBlock<Dollars>,
}

fn share_from_genesis(height: Height, blocks: Count) -> PartsPerMillion32 {
    PartsPerMillion32::from(u64::from(blocks) as f64 / (u64::from(height) + 1) as f64)
}

impl PoolVecs {
    pub(crate) fn new(
        slug: PoolSlug,
        pool_heights: &PoolHeights,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        network: &Windows<&impl ReadableCloneableVec<Height, Hashrate>>,
    ) -> Self {
        let name = |suffix: &str| format!("pool_{slug}_{suffix}");
        let blocks = PoolCumulativeVec::<Count>::new(
            &name("block_count_cumulative_source"),
            slug,
            Column::Blocks,
            pool_heights.clone(),
        );
        let coinbase_sats = PoolCumulativeVec::<Sats>::new(
            &name("rewards_cumulative_sats_source"),
            slug,
            Column::CoinbaseSats,
            pool_heights.clone(),
        );
        let coinbase_cents = PoolCumulativeVec::<Cents>::new(
            &name("rewards_cumulative_cents_source"),
            slug,
            Column::CoinbaseCents,
            pool_heights.clone(),
        );
        let fee_sats = PoolCumulativeVec::<Sats>::new(
            &name("fees_cumulative_sats_source"),
            slug,
            Column::FeeSats,
            pool_heights.clone(),
        );
        let fee_cents = PoolCumulativeVec::<Cents>::new(
            &name("fees_cumulative_cents_source"),
            slug,
            Column::FeeCents,
            pool_heights.clone(),
        );

        let blocks_name = name("block_count");
        let block_count = BlockCount {
            cumulative: LazyPerBlock::from_height_source::<Ident>(
                &format!("{blocks_name}_cumulative"),
                version,
                &blocks,
                mappings,
            ),
            sum: LazyRollingSumsFromHeight::new(
                &format!("{blocks_name}_sum"),
                version,
                &blocks,
                window_starts,
                mappings,
            ),
        };

        let share_name = name("block_share");
        let block_share = BlockShare {
            cumulative: LazyPercentPerBlock::from_height_source(
                &format!("{share_name}_cumulative"),
                version,
                &LazyVec::new(
                    &format!("{share_name}_ppm_source"),
                    version,
                    blocks.read_only_boxed_clone(),
                    share_from_genesis,
                ),
                mappings,
            ),
            windows: LazyPercentRollingWindows::from_cumulative_average(
                &share_name,
                version,
                &blocks,
                window_starts,
                mappings,
            ),
        };

        let rewards_name = name("rewards");
        let rewards = Rewards {
            cumulative: LazyCumulativeValuePerBlock::from_sources(
                &format!("{rewards_name}_cumulative"),
                version,
                &coinbase_sats,
                &coinbase_cents,
                mappings,
            ),
            sum: LazyRollingSumsAmountFromHeight::new(
                &format!("{rewards_name}_sum"),
                version,
                &coinbase_sats,
                &coinbase_cents,
                window_starts,
                mappings,
            ),
        };

        let fees_name = name("fees_per_block");
        let fees_per_block =
            window_starts.zip_with_suffix(&block_count.sum, |suffix, start, blocks| {
                let full_name = format!("{fees_name}_{suffix}");
                let sats = LazyDeltaVec::<Height, Sats, Sats, DeltaSub>::new(
                    &format!("{full_name}_sats_sum"),
                    version,
                    fee_sats.read_only_boxed_clone(),
                    start.read_only_boxed_clone(),
                );
                let cents = LazyDeltaVec::<Height, Cents, Cents, DeltaSub>::new(
                    &format!("{full_name}_cents_sum"),
                    version,
                    fee_cents.read_only_boxed_clone(),
                    start.read_only_boxed_clone(),
                );
                // A window without the pool's blocks holds none of its fees: 0 / 0, undefined.
                let btc = LazyIndexedVec::new(
                    &format!("{full_name}_source"),
                    version,
                    &sats,
                    &blocks.height,
                    |_, sats: Sats, blocks: Count| {
                        Quotient::<Bitcoin>::apply(Bitcoin::from(sats), blocks)
                    },
                );
                let usd = LazyIndexedVec::new(
                    &format!("{full_name}_usd_source"),
                    version,
                    &cents,
                    &blocks.height,
                    |_, cents: Cents, blocks: Count| {
                        Quotient::<Dollars>::apply(Dollars::from(cents), blocks)
                    },
                );
                FeesPerBlock {
                    btc: LazyPerBlock::from_height_source::<Ident>(
                        &full_name, version, &btc, mappings,
                    ),
                    usd: LazyPerBlock::from_height_source::<Ident>(
                        &format!("{full_name}_usd"),
                        version,
                        &usd,
                        mappings,
                    ),
                }
            });

        let hashrate_name = name("hashrate");
        let shares = window_starts.zip_with_suffix(&block_count.sum, |suffix, start, blocks| {
            // The window's exact share of blocks; the published share is rounded to ppm.
            LazyIndexedVec::new(
                &format!("{hashrate_name}_{suffix}_share_source"),
                version,
                &blocks.height,
                *start,
                |height: Height, blocks: Count, start: Height| {
                    let window = u64::from(height) + 1 - u64::from(start);
                    Float64::from(u64::from(blocks) as f64 / window as f64)
                },
            )
        });
        let hashrate = network.zip_with_suffix(&shares, |suffix, network, share| {
            let full_name = format!("{hashrate_name}_{suffix}");
            let source = LazyIndexedVec::new(
                &format!("{full_name}_source"),
                version,
                *network,
                share,
                |_, network: Hashrate, share: Float64| {
                    Hashrate::from(f64::from(network) * f64::from(share))
                },
            );
            LazyPerBlock::from_height_source::<Ident>(&full_name, version, &source, mappings)
        });

        Self {
            block_count,
            block_share,
            rewards,
            fees_per_block,
            hashrate,
        }
    }
}

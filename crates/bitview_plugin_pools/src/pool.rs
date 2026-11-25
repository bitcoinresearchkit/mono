use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, Float64, Hashrate, PartsPerMillion32, PoolSlug};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyCumulativeRolling, LazyCumulativeValuePerBlock, LazyIndexedVec, LazyPerBlock,
    LazyPercentPerBlock, LazyPercentRollingWindows, LazyRollingSumsAmountFromHeight,
    LazyWindowStartVec, PricedRows, ops::Quotient, priced_starts, shapes::Windows,
};
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats};
use vecdb::{
    BinaryTransform, DeltaSub, Ident, LazyDeltaVec, LazyVec, ReadableCloneableVec, Version,
};

use crate::pool_heights::{Column, PoolCumulativeVec, PoolHeights};

#[derive(Clone, Traversable)]
pub struct PoolVecs {
    /// Blocks attributed to the pool.
    #[traversable(lead = "Blocks the pool mined.")]
    pub block_count: LazyCumulativeRolling<Count>,
    /// Share of all blocks attributed to the pool: from genesis through the represented block
    /// (`cumulative`) or over a trailing window.
    #[traversable(lead = "The pool's share of all blocks mined.")]
    block_share: BlockShare,
    /// Coinbase output value of the pool's blocks, with USD valuing each block's coinbase at its
    /// spot price.
    #[traversable(lead = "What the pool's blocks paid it, new coins and fees together.")]
    pub coinbase: Amount,
    /// Transaction fees of the pool's blocks.
    #[traversable(lead = "Fees in the pool's blocks.")]
    fees: PoolFees,
    /// The pool's blocks holding only their coinbase transaction.
    #[traversable(lead = "Empty blocks the pool mined: ones with nothing but the coinbase.")]
    pub empty_block_count: LazyCumulativeRolling<Count>,
    /// The pool's blocks whose coinbase nLockTime is the height minus 1 and whose sequence isn't
    /// final: BIP-54's rule.
    #[traversable(lead = "The pool's blocks following BIP-54's coinbase locktime rule.")]
    bip54_coinbase_locktime_block_count: LazyCumulativeRolling<Count>,
    /// The pool's blocks whose coinbase has a nonzero nLockTime.
    #[traversable(lead = "The pool's blocks with a locktime set on the coinbase.")]
    nonzero_coinbase_locktime_block_count: LazyCumulativeRolling<Count>,
    /// Non-coinbase transactions in the pool's blocks paying a fee whose own fee divided by their
    /// virtual size is under 0.1 sat/vB (zero-fee ones are not counted).
    #[traversable(lead = "Transactions under 0.1 sat/vB in the pool's blocks.")]
    fee_rate_under_0p1_sat_vb_tx_count: LazyCumulativeRolling<Count>,
    /// Estimated hash rate over a trailing window, in hashes per second: the pool's share of the
    /// window's blocks times the network hash-rate estimate for the same window.
    #[traversable(lead = "The pool's hash rate, estimated from its share of the blocks.")]
    hashrate: Windows<LazyPerBlock<Hashrate>>,
}

#[derive(Clone, Traversable)]
#[traversable(client = "Share")]
pub struct BlockShare {
    pub cumulative: LazyPercentPerBlock<PartsPerMillion32>,
    #[traversable(flatten)]
    pub windows: LazyPercentRollingWindows<PartsPerMillion32>,
}

#[derive(Clone, Traversable)]
pub struct Amount {
    /// Through the represented block, from genesis.
    #[traversable(accumulation = Cumulative)]
    pub cumulative: LazyCumulativeValuePerBlock,
    pub sum: LazyRollingSumsAmountFromHeight,
}

#[derive(Clone, Traversable)]
pub struct PoolFees {
    #[traversable(flatten)]
    pub total: Amount,
    /// Average transaction fees of the pool's blocks over a trailing window: the fees of its
    /// blocks in the window divided by their number; null when it mined none.
    #[traversable(lead = "On average, per block.")]
    pub avg: Windows<FeesAvg>,
}

#[derive(Clone, Traversable)]
#[traversable(client = "Value")]
pub struct FeesAvg {
    /// In BTC.
    pub btc: LazyPerBlock<Bitcoin>,
    /// In US dollars, each block's fees valued at its spot price: over the window's blocks from
    /// the first exchange price (block 74,641) on, undefined (null) before it.
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
        let amount_sources = |id: &str, sats: Column, cents: Column| {
            (
                PoolCumulativeVec::<Sats>::new(
                    &name(&format!("{id}_cumulative_sats_source")),
                    slug,
                    sats,
                    pool_heights.clone(),
                ),
                PoolCumulativeVec::<Cents>::new(
                    &name(&format!("{id}_cumulative_cents_source")),
                    slug,
                    cents,
                    pool_heights.clone(),
                ),
            )
        };
        let (coinbase_sats, coinbase_cents) =
            amount_sources("coinbase", Column::CoinbaseSats, Column::CoinbaseCents);
        let (fee_sats, fee_cents) = amount_sources("fees", Column::FeeSats, Column::FeeCents);
        let amount =
            |id: &str, sats: &PoolCumulativeVec<Sats>, cents: &PoolCumulativeVec<Cents>| {
                let name = name(id);
                Amount {
                    cumulative: LazyCumulativeValuePerBlock::from_sources(
                        &format!("{name}_cumulative"),
                        version,
                        sats,
                        cents,
                        mappings,
                    ),
                    sum: LazyRollingSumsAmountFromHeight::new(
                        &format!("{name}_sum"),
                        version,
                        sats,
                        cents,
                        window_starts,
                        mappings,
                    ),
                }
            };
        let count = |id: &str, column: Column| {
            LazyCumulativeRolling::from_cumulative_source(
                &name(id),
                version,
                &PoolCumulativeVec::<Count>::new(
                    &name(&format!("{id}_cumulative_source")),
                    slug,
                    column,
                    pool_heights.clone(),
                ),
                window_starts,
                mappings,
            )
        };

        let block_count = LazyCumulativeRolling::from_cumulative_source(
            &name("block_count"),
            version,
            &blocks,
            window_starts,
            mappings,
        );

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

        let fees_name = name("fees_avg");
        let fees_avg =
            window_starts.zip_with_suffix(&block_count.sum, |suffix, start, block_sum| {
                let full_name = format!("{fees_name}_{suffix}");
                let sats = LazyDeltaVec::<Height, Sats, Sats, DeltaSub>::new(
                    &format!("{full_name}_sats_sum"),
                    version,
                    fee_sats.read_only_boxed_clone(),
                    start.read_only_boxed_clone(),
                );
                // (In dollars, over the window's blocks from the first price on: none before it.)
                let priced = priced_starts(start);
                let cents = LazyDeltaVec::<Height, Cents, Cents, PricedRows<DeltaSub>>::new(
                    &format!("{full_name}_cents_sum"),
                    version,
                    fee_cents.read_only_boxed_clone(),
                    priced.read_only_boxed_clone(),
                );
                let priced_blocks = LazyDeltaVec::<Height, Count, Count, PricedRows<DeltaSub>>::new(
                    &format!("{full_name}_priced_block_count"),
                    version,
                    blocks.read_only_boxed_clone(),
                    priced.read_only_boxed_clone(),
                );
                // A window without the pool's blocks holds none of its fees: 0 / 0, undefined.
                let btc = LazyIndexedVec::new(
                    &format!("{full_name}_source"),
                    version,
                    &sats,
                    &block_sum.height,
                    |_, sats: Sats, blocks: Count| {
                        Quotient::<Bitcoin>::apply(Bitcoin::from(sats), blocks)
                    },
                );
                let usd = LazyIndexedVec::new(
                    &format!("{full_name}_usd_source"),
                    version,
                    &cents,
                    &priced_blocks,
                    |_, cents: Cents, blocks: Count| {
                        Quotient::<Dollars>::apply(Dollars::from(cents), blocks)
                    },
                );
                FeesAvg {
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
            coinbase: amount("coinbase", &coinbase_sats, &coinbase_cents),
            fees: PoolFees {
                total: amount("fees", &fee_sats, &fee_cents),
                avg: fees_avg,
            },
            empty_block_count: count("empty_block_count", Column::EmptyBlocks),
            bip54_coinbase_locktime_block_count: count(
                "bip54_coinbase_locktime_block_count",
                Column::Bip54LocktimeBlocks,
            ),
            nonzero_coinbase_locktime_block_count: count(
                "nonzero_coinbase_locktime_block_count",
                Column::NonzeroLocktimeBlocks,
            ),
            fee_rate_under_0p1_sat_vb_tx_count: count(
                "fee_rate_under_0p1_sat_vb_tx_count",
                Column::Under0p1SatVbTxs,
            ),
            hashrate,
        }
    }
}

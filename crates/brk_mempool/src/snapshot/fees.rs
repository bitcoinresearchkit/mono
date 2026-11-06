use brk_types::{FeeRate, PreciseRecommendedFees, RecommendedFees};

use super::block_stats::BlockStats;

/// Rounding granularity in sat/vB of mempool.space's `/api/v1/fees/precise`.
const PRECISE: FeeRate = FeeRate::from_milli(1);
/// Rounding granularity in sat/vB of mempool.space's `/api/v1/fees/recommended`.
const WHOLE: FeeRate = FeeRate::from_milli(1_000);
/// `getPreciseRecommendedFee` adds this to `fastestFee` and
/// half of it to `halfHourFee`, then floors them. Compensates
/// for sub-1-sat/vB fees mined by hashrate that ignores the
/// relay floor.
const PRIORITY_FACTOR: FeeRate = FeeRate::from_milli(500);
const MIN_FASTEST_FEE: FeeRate = FeeRate::from_milli(1_000);
const MIN_HALF_HOUR_FEE: FeeRate = FeeRate::from_milli(500);
/// At or below this projected-block vsize, the block carries no fee
/// signal and the tier collapses to the floor.
const EMPTY_BLOCK_VSIZE: u64 = 500_000;
/// Above this projected-block vsize, no taper applies. Between
/// `EMPTY_BLOCK_VSIZE` and this threshold, the final-block fee is
/// scaled linearly by `(vsize - EMPTY_BLOCK_VSIZE) / EMPTY_BLOCK_VSIZE`.
const FULL_BLOCK_VSIZE: u64 = 950_000;

/// mempool.space's `getPreciseRecommendedFee` (backend/src/api/fee-api.ts), its tier formula
/// over Bitview's projected blocks. One input differs: a block's median is the vsize-weighted
/// median of its members' effective rates (Core's `getblockstats` rule, as every Bitview median),
/// where mempool.space averages the middle 0.25% of a 4M WU block padded below with its unused
/// space; the two agree on full blocks. `min_fee` is bitcoind's live `mempoolminfee` in sat/vB
/// and floors every tier while the mempool is purging by fee.
pub fn precise(stats: &[BlockStats], min_fee: FeeRate) -> PreciseRecommendedFees {
    let fees = recommend(stats, min_fee, PRECISE);
    PreciseRecommendedFees {
        fastest_fee: (fees.fastest_fee + PRIORITY_FACTOR).max(MIN_FASTEST_FEE),
        half_hour_fee: (fees.half_hour_fee + PRIORITY_FACTOR / 2.0).max(MIN_HALF_HOUR_FEE),
        ..fees
    }
}

/// mempool.space's `getRecommendedFee`: the same tiers at whole sat/vB, without
/// [`precise`]'s priority offsets.
pub fn whole(stats: &[BlockStats], min_fee: FeeRate) -> RecommendedFees {
    let fees = recommend(stats, min_fee, WHOLE);
    let whole = FeeRate::whole_sats;
    RecommendedFees {
        fastest_fee: whole(fees.fastest_fee),
        half_hour_fee: whole(fees.half_hour_fee),
        hour_fee: whole(fees.hour_fee),
        economy_fee: whole(fees.economy_fee),
        minimum_fee: whole(fees.minimum_fee),
    }
}

/// Whole tiers from an earlier view, none below `min_fee`'s floor (mempool.space floors every tier
/// by the live `mempoolminfee`).
pub fn floored_whole(fees: &RecommendedFees, min_fee: FeeRate) -> RecommendedFees {
    let floor = min_fee.ceil_to(WHOLE).max(WHOLE).whole_sats();
    RecommendedFees {
        fastest_fee: fees.fastest_fee.max(floor),
        half_hour_fee: fees.half_hour_fee.max(floor),
        hour_fee: fees.hour_fee.max(floor),
        economy_fee: fees.economy_fee.max(floor),
        minimum_fee: floor,
    }
}

/// [`floored_whole`] for the precise tiers.
pub fn floored_precise(fees: &PreciseRecommendedFees, min_fee: FeeRate) -> PreciseRecommendedFees {
    let floor = min_fee.ceil_to(PRECISE).max(PRECISE);
    PreciseRecommendedFees {
        fastest_fee: fees.fastest_fee.max(floor),
        half_hour_fee: fees.half_hour_fee.max(floor),
        hour_fee: fees.hour_fee.max(floor),
        economy_fee: fees.economy_fee.max(floor),
        minimum_fee: floor,
    }
}

/// mempool.space's `calculateRecommendedFee`: every tier a multiple of `increment`.
fn recommend(stats: &[BlockStats], min_fee: FeeRate, increment: FeeRate) -> PreciseRecommendedFees {
    let minimum_fee = min_fee.ceil_to(increment).max(increment);

    let first = block_fee(stats, 0, None, minimum_fee, increment);
    let second = block_fee(stats, 1, Some(first), minimum_fee, increment);
    let third = block_fee(stats, 2, Some(second), minimum_fee, increment);
    debug_assert!(
        [first, second, third]
            .iter()
            .all(|tier| *tier == tier.ceil_to(increment)),
        "every tier a multiple of the increment"
    );

    let economy = third.clamp(minimum_fee, minimum_fee * 2.0);
    let hour = minimum_fee.max(third).max(economy);
    let half_hour = minimum_fee.max(second).max(hour);
    let fastest = minimum_fee.max(first).max(half_hour);

    PreciseRecommendedFees {
        fastest_fee: fastest,
        half_hour_fee: half_hour,
        hour_fee: hour,
        economy_fee: economy,
        minimum_fee,
    }
}

/// Optimized median for the i-th projected block, or `floor` (the rounded minimum fee) if the
/// block doesn't exist. `prev` is the prior tier's optimized fee, used to smooth toward
/// continuity.
fn block_fee(
    stats: &[BlockStats],
    i: usize,
    prev: Option<FeeRate>,
    floor: FeeRate,
    increment: FeeRate,
) -> FeeRate {
    stats.get(i).map_or(floor, |b| {
        optimize_median_fee(b, stats.get(i + 1), prev, floor, increment)
    })
}

/// Pick the fee for one projected block, smoothing toward the
/// previous tier and discounting partially-full final blocks.
fn optimize_median_fee(
    block: &BlockStats,
    next_block: Option<&BlockStats>,
    previous_fee: Option<FeeRate>,
    floor: FeeRate,
    increment: FeeRate,
) -> FeeRate {
    let median = block.fee_range[3];
    let use_fee = previous_fee.map_or(median, |prev| FeeRate::mean(median, prev));
    // mempool.space's `blockVSize` is the weight's exact quarter: the thresholds compare weight.
    let weight = u64::from(block.total_weight);
    if weight <= 4 * EMPTY_BLOCK_VSIZE || median < floor {
        return floor;
    }
    if weight <= 4 * FULL_BLOCK_VSIZE && next_block.is_none() {
        let multiplier = (weight - 4 * EMPTY_BLOCK_VSIZE) as f64 / (4 * EMPTY_BLOCK_VSIZE) as f64;
        return (use_fee * multiplier).round_to(increment).max(floor);
    }
    use_fee.ceil_to(increment).max(floor)
}

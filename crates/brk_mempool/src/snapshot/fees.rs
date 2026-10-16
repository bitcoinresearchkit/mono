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
/// signal and the tier collapses to `min_fee`.
const EMPTY_BLOCK_VSIZE: u64 = 500_000;
/// Above this projected-block vsize, no taper applies. Between
/// `EMPTY_BLOCK_VSIZE` and this threshold, the final-block fee is
/// scaled linearly by `(vsize - EMPTY_BLOCK_VSIZE) / EMPTY_BLOCK_VSIZE`.
const FULL_BLOCK_VSIZE: u64 = 950_000;

/// Literal port of mempool.space's `getPreciseRecommendedFee`
/// (backend/src/api/fee-api.ts). `min_fee` is bitcoind's live
/// `mempoolminfee` in sat/vB and acts as a floor for every tier
/// while the mempool is purging by fee.
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
    let whole = |rate: FeeRate| f64::from(rate) as u64;
    RecommendedFees {
        fastest_fee: whole(fees.fastest_fee),
        half_hour_fee: whole(fees.half_hour_fee),
        hour_fee: whole(fees.hour_fee),
        economy_fee: whole(fees.economy_fee),
        minimum_fee: whole(fees.minimum_fee),
    }
}

/// mempool.space's `calculateRecommendedFee`: every tier a multiple of `increment`.
fn recommend(stats: &[BlockStats], min_fee: FeeRate, increment: FeeRate) -> PreciseRecommendedFees {
    let minimum_fee = min_fee.ceil_to(increment).max(increment);

    let first = block_fee(stats, 0, None, minimum_fee, increment);
    let second = block_fee(stats, 1, Some(first), minimum_fee, increment);
    let third = block_fee(stats, 2, Some(second), minimum_fee, increment);

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

/// Optimized median for the i-th projected block, or `min_fee` if
/// the block doesn't exist. `prev` is the prior tier's optimized
/// fee, used to smooth toward continuity.
fn block_fee(
    stats: &[BlockStats],
    i: usize,
    prev: Option<FeeRate>,
    min_fee: FeeRate,
    increment: FeeRate,
) -> FeeRate {
    stats.get(i).map_or(min_fee, |b| {
        optimize_median_fee(b, stats.get(i + 1), prev, min_fee, increment)
    })
}

/// Pick the fee for one projected block, smoothing toward the
/// previous tier and discounting partially-full final blocks.
fn optimize_median_fee(
    block: &BlockStats,
    next_block: Option<&BlockStats>,
    previous_fee: Option<FeeRate>,
    min_fee: FeeRate,
    increment: FeeRate,
) -> FeeRate {
    let median = block.fee_range[3];
    let use_fee = previous_fee.map_or(median, |prev| FeeRate::mean(median, prev));
    let vsize = u64::from(block.total_vsize);
    if vsize <= EMPTY_BLOCK_VSIZE || median < min_fee {
        return min_fee;
    }
    if vsize <= FULL_BLOCK_VSIZE && next_block.is_none() {
        let multiplier = (vsize - EMPTY_BLOCK_VSIZE) as f64 / EMPTY_BLOCK_VSIZE as f64;
        return (use_fee * multiplier).round_to(increment).max(min_fee);
    }
    use_fee.ceil_to(increment).max(min_fee)
}

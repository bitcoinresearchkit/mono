use std::sync::Arc;

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

/// The recommended tiers and the blocks they stand on.
#[derive(Default)]
pub(crate) struct FeeView {
    /// The view's own blocks when it is complete. An incomplete view holds back part of the pool,
    /// so its blocks after block 0 would quote too low: the last view's stand in for them, and
    /// for its block 0 too unless that is Core's template.
    pub stats: Arc<[BlockStats]>,
    pub whole: RecommendedFees,
    pub precise: PreciseRecommendedFees,
    /// Whether a complete view stands behind the tiers. False only before the first one.
    pub complete: bool,
}

impl FeeView {
    /// `held`: an incomplete view's predecessor, and whether its own block 0 is Core's whole
    /// template.
    pub fn new(blocks: &[BlockStats], held: Option<(&FeeView, bool)>, min_fee: FeeRate) -> Self {
        let (stats, complete): (Arc<[BlockStats]>, bool) = match held {
            None => (blocks.into(), true),
            Some((previous, template)) => {
                let own = usize::from(template).min(blocks.len());
                let stats = blocks[..own]
                    .iter()
                    .chain(previous.stats.iter().skip(own))
                    .cloned()
                    .collect();
                (stats, previous.complete)
            }
        };
        let precise = precise(&stats, min_fee);
        Self {
            whole: whole(&precise),
            precise,
            stats,
            complete,
        }
    }
}

/// Queue-based recommendations with fixed, target-specific empirical margins. The base is
/// mempool.space's tier formula over Bitview's member-vsize-weighted chunk medians (different
/// from mempool.space's capacity-padded central band). The margin is not a probability promise.
pub fn precise(stats: &[BlockStats], min_fee: FeeRate) -> PreciseRecommendedFees {
    let fees = recommend(stats, min_fee, PRECISE);
    with_margin(PreciseRecommendedFees {
        fastest_fee: (fees.fastest_fee + PRIORITY_FACTOR).max(MIN_FASTEST_FEE),
        half_hour_fee: (fees.half_hour_fee + PRIORITY_FACTOR / 2.0).max(MIN_HALF_HOUR_FEE),
        ..fees
    })
}

/// One curve for both representations; whole quotes round up without recomputing the model.
fn whole(fees: &PreciseRecommendedFees) -> RecommendedFees {
    let whole = |rate: FeeRate| {
        debug_assert_ne!(rate.milli(), u64::MAX, "a finite fee quote");
        // Round the integer directly: a rounded-up FeeRate could overflow at the finite bound.
        rate.milli().div_ceil(WHOLE.milli())
    };
    RecommendedFees {
        fastest_fee: whole(fees.fastest_fee),
        half_hour_fee: whole(fees.half_hour_fee),
        hour_fee: whole(fees.hour_fee),
        economy_fee: whole(fees.economy_fee),
        minimum_fee: whole(fees.minimum_fee),
    }
}

/// Ratios fitted to the .95 quantile of required/base on the development capture, with every
/// label matured before the holdout. Targets are 1, 3, 6 and 144 actual blocks. The capture
/// does not support a long-target fit, so economy retains its base. No runtime learning or
/// historical-confirmation guarantee is implied by these fixed margins.
pub(crate) fn with_margin(fees: PreciseRecommendedFees) -> PreciseRecommendedFees {
    const MARGINS: [(u64, u64); 4] = [(2006, 1000), (1011, 500), (1000, 341), (1, 1)];
    let mut rates = [
        fees.fastest_fee,
        fees.half_hour_fee,
        fees.hour_fee,
        fees.economy_fee,
    ];
    for (rate, (numerator, denominator)) in rates.iter_mut().zip(MARGINS) {
        let milli = (u128::from(rate.milli()) * u128::from(numerator))
            .div_ceil(u128::from(denominator))
            // FeeRate reserves u64::MAX as NaN; its finite-bound constant is private.
            .min(u128::from(u64::MAX - 1));
        *rate = FeeRate::from_milli(milli as u64).max(fees.minimum_fee);
    }
    for i in (0..rates.len() - 1).rev() {
        rates[i] = rates[i].max(rates[i + 1]);
    }
    PreciseRecommendedFees {
        fastest_fee: rates[0],
        half_hour_fee: rates[1],
        hour_fee: rates[2],
        economy_fee: rates[3],
        ..fees
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
    // mempool.space's `blockVSize` is the weight's exact quarter: the thresholds compare weight.
    let weight = u64::from(block.total_weight);
    if weight <= 4 * EMPTY_BLOCK_VSIZE || median < floor {
        return floor;
    }
    if weight <= 4 * FULL_BLOCK_VSIZE && next_block.is_none() {
        // `use_fee * (weight - empty) / empty`, rounded once to the increment (half up, as
        // `Math.round`): exactly, in milli-sats, from twice the mean so it never rounds before.
        let twice = u128::from(median.milli()) + u128::from(previous_fee.unwrap_or(median).milli());
        let empty = u128::from(4 * EMPTY_BLOCK_VSIZE);
        let step = 2 * empty * u128::from(increment.milli());
        let units = (2 * twice * u128::from(weight - 4 * EMPTY_BLOCK_VSIZE) + step) / (2 * step);
        // A fraction of the mean of two fee rates, rounded up a step at most: it fits.
        let milli = u64::try_from(units * u128::from(increment.milli())).unwrap_or(u64::MAX);
        return FeeRate::from_milli(milli).max(floor);
    }
    let use_fee = previous_fee.map_or(median, |prev| FeeRate::mean(median, prev));
    use_fee.ceil_to(increment).max(floor)
}

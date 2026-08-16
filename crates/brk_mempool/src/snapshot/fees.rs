use brk_types::{FeeRate, RecommendedFees};

use super::block_stats::BlockStats;

/// Output rounding granularity in sat/vB. mempool.space's
/// `/api/v1/fees/recommended` uses `1.0`, their `/precise`
/// variant uses `0.001`. bitview always emits precise.
const MIN_INCREMENT: FeeRate = FeeRate::from_milli(1);
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

pub struct Fees;

impl Fees {
    /// Literal port of mempool.space's `getPreciseRecommendedFee`
    /// (backend/src/api/fee-api.ts). `min_fee` is bitcoind's live
    /// `mempoolminfee` in sat/vB and acts as a floor for every tier
    /// while the mempool is purging by fee.
    pub fn compute(stats: &[BlockStats], min_fee: FeeRate) -> RecommendedFees {
        let minimum_fee = min_fee.ceil_to(MIN_INCREMENT).max(MIN_INCREMENT);

        let first = Self::block_fee(stats, 0, None, minimum_fee);
        let second = Self::block_fee(stats, 1, Some(first), minimum_fee);
        let third = Self::block_fee(stats, 2, Some(second), minimum_fee);

        let economy = third.clamp(minimum_fee, minimum_fee * 2.0);
        let hour = minimum_fee.max(third).max(economy);
        let half_hour = minimum_fee.max(second).max(hour);
        let fastest = minimum_fee.max(first).max(half_hour);

        let fastest = (fastest + PRIORITY_FACTOR).max(MIN_FASTEST_FEE);
        let half_hour = (half_hour + PRIORITY_FACTOR / 2.0).max(MIN_HALF_HOUR_FEE);

        RecommendedFees {
            fastest_fee: fastest.round_milli(),
            half_hour_fee: half_hour.round_milli(),
            hour_fee: hour.round_milli(),
            economy_fee: economy.round_milli(),
            minimum_fee: minimum_fee.round_milli(),
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
    ) -> FeeRate {
        stats.get(i).map_or(min_fee, |b| {
            Self::optimize_median_fee(b, stats.get(i + 1), prev, min_fee)
        })
    }

    /// Pick the fee for one projected block, smoothing toward the
    /// previous tier and discounting partially-full final blocks.
    fn optimize_median_fee(
        block: &BlockStats,
        next_block: Option<&BlockStats>,
        previous_fee: Option<FeeRate>,
        min_fee: FeeRate,
    ) -> FeeRate {
        let median = block.fee_range[3];
        let use_fee = previous_fee.map_or(median, |prev| FeeRate::mean(median, prev));
        let vsize = u64::from(block.total_vsize);
        if vsize <= EMPTY_BLOCK_VSIZE || median < min_fee {
            return min_fee;
        }
        if vsize <= FULL_BLOCK_VSIZE && next_block.is_none() {
            let multiplier = (vsize - EMPTY_BLOCK_VSIZE) as f64 / EMPTY_BLOCK_VSIZE as f64;
            return (use_fee * multiplier).round_to(MIN_INCREMENT).max(min_fee);
        }
        use_fee.ceil_to(MIN_INCREMENT).max(min_fee)
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Sats, VSize};

    use super::*;

    fn block(vsize: u64, median_fee: f64) -> BlockStats {
        let median = FeeRate::new(median_fee);
        BlockStats {
            tx_count: 1,
            total_size: vsize,
            total_vsize: VSize::from(vsize),
            total_fee: Sats::from((vsize as f64 * median_fee) as u64),
            fee_range: [median, median, median, median, median, median, median],
        }
    }

    #[test]
    fn empty_stats_collapses_every_tier_to_min_fee() {
        let min = FeeRate::new(2.0);
        let fees = Fees::compute(&[], min);
        let priority_fastest = FeeRate::new(2.5); // min + PRIORITY_FACTOR
        let priority_half_hour = FeeRate::new(2.25); // min + PRIORITY_FACTOR/2
        assert_eq!(fees.minimum_fee, min);
        assert_eq!(fees.economy_fee, min);
        assert_eq!(fees.hour_fee, min);
        assert_eq!(fees.fastest_fee, priority_fastest);
        assert_eq!(fees.half_hour_fee, priority_half_hour);
    }

    #[test]
    fn min_fee_floor_lifts_below_one_sat_rates() {
        // `mempoolminfee` below MIN_INCREMENT: result is clamped up.
        let min = FeeRate::new(0.0);
        let fees = Fees::compute(&[], min);
        assert!(f64::from(fees.minimum_fee) >= 0.001);
        // `fastest_fee` always at least MIN_FASTEST_FEE.
        assert!(f64::from(fees.fastest_fee) >= 1.0);
    }

    #[test]
    fn small_partial_final_block_collapses_to_min_fee() {
        // vsize <= EMPTY_BLOCK_VSIZE: returns min_fee unconditionally.
        let stats = vec![block(400_000, 12.5)];
        let min = FeeRate::new(1.0);
        let fees = Fees::compute(&stats, min);
        assert_eq!(fees.hour_fee, min);
        assert_eq!(fees.economy_fee, min);
    }

    #[test]
    fn full_block_carries_signal_into_top_tier() {
        let stats = vec![block(1_000_000, 25.0), block(1_000_000, 10.0)];
        let min = FeeRate::new(1.0);
        let fees = Fees::compute(&stats, min);
        // fastest gets PRIORITY_FACTOR (0.5) added.
        assert_eq!(fees.fastest_fee, FeeRate::new(25.5));
        // hour comes from block[2], which doesn't exist -> collapses to min.
        assert_eq!(fees.hour_fee, min);
    }

    #[test]
    fn partial_final_block_tapers_linearly() {
        // vsize in (EMPTY, FULL]: rate = use_fee * (vsize - EMPTY)/EMPTY.
        // 725_000 vsize -> multiplier = 225_000 / 500_000 = 0.45.
        let stats = vec![block(725_000, 10.0)];
        let min = FeeRate::new(1.0);
        let fees = Fees::compute(&stats, min);
        // economy/hour come from the same (only) block, both tapered.
        // 10.0 * 0.45 = 4.5, fastest = 4.5 + 0.5 = 5.0.
        assert_eq!(fees.fastest_fee, FeeRate::new(5.0));
    }
}

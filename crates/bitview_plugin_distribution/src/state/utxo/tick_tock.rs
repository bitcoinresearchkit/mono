use brk_cohort::{AGE_BOUNDARIES, AGE_RANGE_IDS, AgeRange, AgeRangeId};
use brk_types::{
    CostBasisSnapshot, ONE_DAY_IN_SEC_F64, ONE_HOUR_IN_SEC, Sats, StoredF64, Timestamp,
};
use vecdb::unlikely;

use crate::state::BlockState;

use super::UTXOStates;

#[derive(Default)]
pub struct TickTockResult {
    pub matured: AgeRange<Sats>,
    pub coindays_created: AgeRange<StoredF64>,
}

impl UTXOStates {
    /// Handle age transitions when processing a new block.
    ///
    /// UTXOs age with each block. When they cross hour boundaries,
    /// they move between age-based cohorts (e.g., from "0-1h" to "1h-1d").
    ///
    /// Uses cached positions per boundary to avoid binary search.
    /// Since timestamps are monotonic, positions only advance forward.
    /// Complexity: O(k * c), where k is the boundary count and c is ~1 forward scan step.
    ///
    /// Returns both the sats matured out of each cohort and the coindays created
    /// inside each cohort during the preceding block interval.
    fn tick_tock_next_block(
        &mut self,
        chain_state: &[BlockState],
        timestamp: Timestamp,
    ) -> TickTockResult {
        if chain_state.is_empty() {
            return TickTockResult::default();
        }

        let prev_timestamp = chain_state.last().unwrap().timestamp;
        let elapsed = (*timestamp).saturating_sub(*prev_timestamp);

        // Skip if no time has passed
        if elapsed == 0 {
            return TickTockResult::default();
        }

        let mut matured = AgeRange::default();
        let Self {
            age_range,
            transient,
            ..
        } = self;
        let cached = &mut transient.tick_tock_cached_positions;

        // Every live sat creates time in its cohort at the start of the interval.
        // Boundary crossings below transfer only the post-crossing tail to the
        // next cohort, which keeps the allocation exact for long block gaps too.
        let mut created_sat_seconds = AgeRange::from_fn(|id| {
            u128::from(id.select(age_range).supply.value) * u128::from(elapsed)
        });
        let expected_sat_seconds = created_sat_seconds.iter().copied().sum::<u128>();

        // For each boundary (in hours), find blocks that just crossed it
        for (boundary_idx, (&boundary_hours, adjacent)) in AGE_BOUNDARIES
            .iter()
            .zip(AGE_RANGE_IDS.windows(2))
            .enumerate()
        {
            let boundary_seconds = (boundary_hours as u32) * ONE_HOUR_IN_SEC;

            // Blocks crossing boundary B have timestamps in (prev - B*HOUR, curr - B*HOUR]
            // prev_hours < B and curr_hours >= B
            // means: block was younger than B hours, now is B hours or older
            let upper_timestamp = (*timestamp).saturating_sub(boundary_seconds);
            let lower_timestamp = (*prev_timestamp).saturating_sub(boundary_seconds);

            // Skip if the range is empty (would happen if boundary > chain age)
            if upper_timestamp <= lower_timestamp {
                continue;
            }

            // Find start_idx: use cached position + forward scan (O(1) typical).
            // On first call after restart, cached is 0 so fall back to binary search.
            let start_idx = if unlikely(cached[boundary_idx] == 0 && chain_state.len() > 1) {
                let idx = chain_state.partition_point(|b| *b.timestamp <= lower_timestamp);
                cached[boundary_idx] = idx;
                idx
            } else {
                let mut idx = cached[boundary_idx];
                while idx < chain_state.len() && *chain_state[idx].timestamp <= lower_timestamp {
                    idx += 1;
                }
                cached[boundary_idx] = idx;
                idx
            };

            // Linear scan for end (typically 0-2 blocks past start)
            let end_idx = chain_state[start_idx..]
                .iter()
                .position(|b| *b.timestamp > upper_timestamp)
                .map_or(chain_state.len(), |pos| start_idx + pos);

            // Move supply from younger cohort to older cohort
            let younger = adjacent[0];
            let older = adjacent[1];
            let matured_sats = younger.select_mut(&mut matured);
            for block_state in &chain_state[start_idx..end_idx] {
                let crossing_timestamp = (*block_state.timestamp).saturating_add(boundary_seconds);
                let tail_seconds = (*timestamp).saturating_sub(crossing_timestamp);
                debug_assert!(tail_seconds <= elapsed);
                move_created_tail(
                    &mut created_sat_seconds,
                    younger,
                    older,
                    block_state.supply.value,
                    tail_seconds,
                );

                let snapshot = CostBasisSnapshot::from_utxo(block_state.price, &block_state.supply);
                younger.select_mut(age_range).decrement_snapshot(&snapshot);
                older.select_mut(age_range).increment_snapshot(&snapshot);
                *matured_sats += block_state.supply.value;
            }
        }

        debug_assert_eq!(
            created_sat_seconds.iter().copied().sum::<u128>(),
            expected_sat_seconds,
        );

        TickTockResult {
            matured,
            coindays_created: AgeRange::from_fn(|id| {
                sat_seconds_to_coindays(*id.select(&created_sat_seconds))
            }),
        }
    }
}

pub fn tick_tock_next_block(
    states: &mut UTXOStates,
    chain_state: &[BlockState],
    timestamp: Timestamp,
) -> TickTockResult {
    states.tick_tock_next_block(chain_state, timestamp)
}

#[inline(always)]
fn move_created_tail(
    created_sat_seconds: &mut AgeRange<u128>,
    younger: AgeRangeId,
    older: AgeRangeId,
    sats: Sats,
    tail_seconds: u32,
) {
    let correction = u128::from(sats) * u128::from(tail_seconds);
    let younger_value = younger.select_mut(created_sat_seconds);
    *younger_value = younger_value
        .checked_sub(correction)
        .expect("created sat-seconds underflow");
    *older.select_mut(created_sat_seconds) += correction;
}

#[inline(always)]
fn sat_seconds_to_coindays(sat_seconds: u128) -> StoredF64 {
    StoredF64::from(sat_seconds as f64 / Sats::ONE_BTC_U128 as f64 / ONE_DAY_IN_SEC_F64)
}

#[cfg(test)]
mod tests {
    use brk_types::ONE_DAY_IN_SEC;

    use super::*;

    #[test]
    fn created_time_is_split_at_the_exact_age_boundary() {
        let mut created_sat_seconds = AgeRange {
            under_1h: u128::from(Sats::ONE_BTC) * 10 * 60,
            ..Default::default()
        };

        move_created_tail(
            &mut created_sat_seconds,
            AgeRangeId::Under1H,
            AgeRangeId::From1HTo1D,
            Sats::ONE_BTC,
            9 * 60,
        );

        assert_eq!(created_sat_seconds.under_1h, u128::from(Sats::ONE_BTC) * 60,);
        assert_eq!(
            created_sat_seconds._1h_to_1d,
            u128::from(Sats::ONE_BTC) * 9 * 60,
        );
        assert_eq!(
            created_sat_seconds.iter().copied().sum::<u128>(),
            u128::from(Sats::ONE_BTC) * 10 * 60,
        );
    }

    #[test]
    fn sat_seconds_are_converted_to_fractional_coindays() {
        let one_hour = f64::from(sat_seconds_to_coindays(
            u128::from(Sats::ONE_BTC) * u128::from(ONE_HOUR_IN_SEC),
        ));

        assert!((one_hour - 1.0 / 24.0).abs() < 1e-12);
    }

    #[test]
    fn successive_crossings_split_a_long_interval() {
        let interval = 2 * ONE_DAY_IN_SEC;
        let mut created_sat_seconds = AgeRange {
            under_1h: u128::from(Sats::ONE_BTC) * u128::from(interval),
            ..Default::default()
        };

        move_created_tail(
            &mut created_sat_seconds,
            AgeRangeId::Under1H,
            AgeRangeId::From1HTo1D,
            Sats::ONE_BTC,
            interval - ONE_HOUR_IN_SEC,
        );
        move_created_tail(
            &mut created_sat_seconds,
            AgeRangeId::From1HTo1D,
            AgeRangeId::From1DTo1W,
            Sats::ONE_BTC,
            interval - ONE_DAY_IN_SEC,
        );

        assert_eq!(
            created_sat_seconds.under_1h,
            u128::from(Sats::ONE_BTC) * u128::from(ONE_HOUR_IN_SEC),
        );
        assert_eq!(
            created_sat_seconds._1h_to_1d,
            u128::from(Sats::ONE_BTC) * u128::from(ONE_DAY_IN_SEC - ONE_HOUR_IN_SEC),
        );
        assert_eq!(
            created_sat_seconds._1d_to_1w,
            u128::from(Sats::ONE_BTC) * u128::from(ONE_DAY_IN_SEC),
        );
    }
}

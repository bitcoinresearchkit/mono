use bitview_cohort::{AgeRange, AgeRangeId, for_each_age_crossing};
use bitview_primitives::{CostBasisSnapshot, StoredF64};
use brk_types::{ONE_DAY_IN_SEC_F64, Sats, Timestamp};
use statedb::Amount;

use super::UTXOStates;
use crate::{compute::ComputeContext, state::supply};

#[derive(Default)]
pub struct TickTockResult {
    pub matured: AgeRange<Sats>,
    pub coindays_created: AgeRange<StoredF64>,
}

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
pub fn tick_tock_next_block(
    states: &mut UTXOStates,
    amounts: &[Amount],
    ctx: &ComputeContext<'_>,
    timestamp: Timestamp,
) -> TickTockResult {
    if amounts.is_empty() {
        return TickTockResult::default();
    }

    let timestamps = &ctx.height_to_timestamp[..amounts.len()];
    let prev_timestamp = *timestamps.last().unwrap();
    let elapsed = (*timestamp).saturating_sub(*prev_timestamp);

    // Skip if no time has passed
    if elapsed == 0 {
        return TickTockResult::default();
    }

    let mut matured = AgeRange::default();
    let UTXOStates {
        age_range,
        transient,
        ..
    } = states;
    let cached = &mut transient.tick_tock_cached_positions;

    // Every live sat creates time in its cohort at the start of the interval.
    // Boundary crossings below transfer only the post-crossing tail to the
    // next cohort, which keeps the allocation exact for long block gaps too.
    let mut created_sat_seconds =
        AgeRange::from_fn(|id| u128::from(id.select(age_range).supply.value) * u128::from(elapsed));
    let expected_sat_seconds = created_sat_seconds.iter().copied().sum::<u128>();

    for_each_age_crossing(
        timestamps,
        timestamp,
        cached,
        |h, younger, older, crossing_timestamp| {
            let supply = supply(amounts[h]);
            let tail_seconds = (*timestamp).saturating_sub(crossing_timestamp);
            debug_assert!(tail_seconds <= elapsed);
            move_created_tail(
                &mut created_sat_seconds,
                younger,
                older,
                supply.value,
                tail_seconds,
            );
            let snapshot = CostBasisSnapshot::from_utxo(ctx.height_to_price[h], &supply);
            younger.select_mut(age_range).decrement_snapshot(&snapshot);
            older.select_mut(age_range).increment_snapshot(&snapshot);
            *younger.select_mut(&mut matured) += supply.value;
        },
    );

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

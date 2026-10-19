use bitview_cohort::{AgeRange, AgeRangeId, for_each_age_crossing};
use bitview_primitives::{CoinDays, CostBasisSnapshot};
use brk_types::{ONE_DAY_IN_SEC_F64, Sats, Timestamp};
use statedb::Amount;

use super::UTXOStates;
use crate::{compute::ComputeContext, state::supply};

#[derive(Default)]
pub struct TickTockResult {
    pub matured: AgeRange<Sats>,
    pub coindays_created: AgeRange<CoinDays>,
}

/// Moves the origins that crossed an age boundary since the previous block into the older
/// range. Each boundary's search position is cached and, timestamps being monotonic, only
/// advances, so a block scans just the origins crossing it.
///
/// Returns the sats matured out of each range and the coin days created in each during the
/// preceding block interval.
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
    let elapsed = *timestamp - *prev_timestamp;
    if elapsed == 0 {
        return TickTockResult::default();
    }

    let mut matured = AgeRange::default();
    let UTXOStates {
        age_range,
        tick_tock_cached_positions: cached,
        ..
    } = states;

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
            let tail_seconds = *timestamp - crossing_timestamp;
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
    debug_assert!(
        *younger_value >= correction,
        "created sat-seconds underflow"
    );
    *younger_value -= correction;
    *older.select_mut(created_sat_seconds) += correction;
}

#[inline(always)]
fn sat_seconds_to_coindays(sat_seconds: u128) -> CoinDays {
    CoinDays::from(sat_seconds as f64 / Sats::ONE_BTC_U128 as f64 / ONE_DAY_IN_SEC_F64)
}

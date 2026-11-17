use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_distribution::walk::Block;
use bitview_primitives::{CoinDays, CostBasisSnapshot};
use brk_types::{ONE_DAY_IN_SEC_F64, Sats};

use super::UTXOStates;
use crate::state::supply;

#[derive(Default)]
pub struct TickTockResult {
    pub aged_out: AgeRange<Sats>,
    pub coindays_created: AgeRange<CoinDays>,
}

/// Moves the origins that crossed an age boundary since the previous block into the older
/// range, as they stood before `block`.
///
/// Returns the sats that aged out of each range and the coin days created in each during the
/// preceding block interval.
pub fn tick_tock(states: &mut UTXOStates, block: &Block<'_>) -> TickTockResult {
    let Some(previous) = block.height.checked_sub(1) else {
        return TickTockResult::default();
    };
    let timestamp = block.timestamp();
    let elapsed = *timestamp - *block.timestamps[previous];
    if elapsed == 0 {
        return TickTockResult::default();
    }

    let mut aged_out = AgeRange::default();
    let age_range = &mut states.age_range;

    // Every live sat creates time in its cohort at the start of the interval.
    // Boundary crossings below transfer only the post-crossing tail to the
    // next cohort, which keeps the allocation exact for long block gaps too.
    let mut created_sat_seconds =
        AgeRange::from_fn(|id| u128::from(id.select(age_range).supply.value) * u128::from(elapsed));
    let expected_sat_seconds = created_sat_seconds.iter().copied().sum::<u128>();

    for crossing in block.crossings {
        let (younger, older) = (crossing.younger, crossing.older);
        let supply = supply(crossing.amount);
        let tail_seconds = *timestamp - crossing.at;
        debug_assert!(tail_seconds <= elapsed);
        move_created_tail(
            &mut created_sat_seconds,
            younger,
            older,
            supply.value,
            tail_seconds,
        );
        let snapshot = CostBasisSnapshot::from_utxo(block.prices[crossing.origin], &supply);
        younger.select_mut(age_range).decrement_snapshot(&snapshot);
        older.select_mut(age_range).increment_snapshot(&snapshot);
        *younger.select_mut(&mut aged_out) += supply.value;
    }

    debug_assert_eq!(
        created_sat_seconds.iter().copied().sum::<u128>(),
        expected_sat_seconds,
    );

    TickTockResult {
        aged_out,
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

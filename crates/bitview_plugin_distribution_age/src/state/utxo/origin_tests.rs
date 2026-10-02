use bitview_cohort::AgeRangeId;
use brk_types::{Cents, Height, ONE_DAY_IN_SEC, Sats, SupplyState, Timestamp};
use statedb::Amount;

use crate::{
    compute::{ComputeContext, PriceRangeMax},
    state::{RealizedOps, UTXOStates, tick_tock_next_block},
};

#[test]
fn origins_age_across_a_long_gap_before_spending_including_zero_value_outputs() {
    let timestamps =
        [0, 1800, 2 * ONE_DAY_IN_SEC].map(|elapsed| Timestamp::new(1_700_000_000 + elapsed));
    let prices = [100, 150, 200].map(Cents::new);
    let mut peak = PriceRangeMax::default();
    peak.extend(&prices);
    let ctx = ComputeContext {
        starting_height: Height::ZERO,
        last_height: Height::new(2),
        height_to_timestamp: &timestamps,
        height_to_price: &prices,
        price_range_max: &peak,
    };
    let funded = SupplyState {
        value: Sats::ONE_BTC * 2u64,
        utxo_count: 2,
    };
    let zero = SupplyState {
        value: Sats::ZERO,
        utxo_count: 3,
    };
    let amounts = [
        Amount {
            sats: u64::from(funded.value),
            count: 2,
        },
        Amount { sats: 0, count: 3 },
    ];
    let mut states = UTXOStates::new();
    states.reset().unwrap();
    states.receive_origins(funded, Height::ZERO, timestamps[0], prices[0]);
    states.apply_pending();
    states.reset_block();
    let first = tick_tock_next_block(&mut states, &amounts[..1], &ctx, timestamps[1]);
    assert!((f64::from(first.coindays_created.under_1h) - 1.0 / 24.0).abs() < 1e-12);
    states.receive_origins(zero, Height::new(1), timestamps[1], prices[1]);
    states.apply_pending();
    states.reset_block();

    let tick = tick_tock_next_block(&mut states, &amounts, &ctx, timestamps[2]);
    for &id in AgeRangeId::ALL {
        let created = match id {
            AgeRangeId::Under1H => 1.0 / 24.0,
            AgeRangeId::From1HTo1D => 23.0 / 12.0,
            AgeRangeId::From1DTo1W => 2.0,
            _ => 0.0,
        };
        assert!((f64::from(*id.select(&tick.coindays_created)) - created).abs() < 1e-12);
        let cohort = id.select(&states.age_range);
        let active = id == AgeRangeId::From1DTo1W;
        assert_eq!(
            (cohort.supply.value, cohort.supply.utxo_count),
            if active {
                (funded.value, 5)
            } else {
                (Sats::ZERO, 0)
            }
        );
        assert_eq!(
            cohort.realized.cap(),
            if active { Cents::new(200) } else { Cents::ZERO }
        );
    }
    assert_eq!(tick.matured.under_1h, funded.value);
    assert_eq!(tick.matured._1h_to_1d, funded.value);
    let satblocks = states.send_origins(
        [(Height::ZERO, funded), (Height::new(1), zero)],
        Height::new(2),
        &ctx,
    );
    assert_eq!(satblocks, 4 * Sats::ONE_BTC_U128);
    states.apply_pending();
    for spent in [
        states
            .age_range
            .iter()
            .map(|s| u64::from(s.output_counts().1))
            .sum::<u64>(),
        states
            .epoch
            .iter()
            .map(|s| u64::from(s.output_counts().1))
            .sum(),
        states
            .class
            .iter()
            .map(|s| u64::from(s.output_counts().1))
            .sum(),
    ] {
        assert_eq!(spent, 5);
    }
    for (supply, cap) in states
        .age_range
        .iter()
        .map(|s| (s.supply.value, s.realized.cap()))
        .chain(
            states
                .epoch
                .iter()
                .map(|s| (s.supply.value, s.realized.cap())),
        )
        .chain(
            states
                .class
                .iter()
                .map(|s| (s.supply.value, s.realized.cap())),
        )
    {
        assert_eq!(supply, Sats::ZERO);
        assert_eq!(cap, Cents::ZERO);
    }
    assert!(states.age_range.iter().all(|s| s.supply.utxo_count == 0));
    assert!(states.epoch.iter().all(|s| s.supply.utxo_count == 0));
    assert!(states.class.iter().all(|s| s.supply.utxo_count == 0));
    assert_eq!(
        states
            .age_range
            .iter()
            .map(|s| s.realized.profit().inner())
            .sum::<u64>(),
        200
    );
    assert_eq!(
        states
            .age_range
            .iter()
            .map(|s| u64::from(s.transfer_volume()))
            .sum::<u64>(),
        u64::from(funded.value)
    );
}

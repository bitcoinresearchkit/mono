use crate::{
    compute::{ComputeContext, PriceRangeMax},
    state::UTXOStates,
};
use brk_types::{Cents, Height, Sats, SupplyState, Timestamp};
#[test]
fn aggregated_origins_match_individual_spends() {
    let mut grouped = UTXOStates::new();
    let mut individual = UTXOStates::new();
    grouped.reset().unwrap();
    individual.reset().unwrap();
    let price = Cents::new(10_000);
    let timestamp = Timestamp::new(1_700_000_000);
    let supply = SupplyState {
        value: Sats::ONE_BTC * 2u64,
        utxo_count: 2,
    };
    grouped.receive_origins(supply, Height::ZERO, timestamp, price);
    individual.receive_origins(supply, Height::ZERO, timestamp, price);
    let mut peak = PriceRangeMax::default();
    peak.extend(&[price; 3]);
    let ctx = ComputeContext {
        starting_height: Height::ZERO,
        last_height: Height::new(2),
        height_to_timestamp: &[timestamp; 3],
        height_to_price: &[price; 3],
        price_range_max: &peak,
    };
    let grouped_blocks = grouped.send_origins([(Height::ZERO, supply)], Height::new(2), &ctx);
    let individual_blocks = individual.send_origins(
        [
            (
                Height::ZERO,
                SupplyState {
                    value: Sats::ONE_BTC,
                    utxo_count: 1,
                },
            ),
            (
                Height::ZERO,
                SupplyState {
                    value: Sats::ONE_BTC,
                    utxo_count: 1,
                },
            ),
        ],
        Height::new(2),
        &ctx,
    );
    assert_eq!(grouped_blocks, 4 * Sats::ONE_BTC_U128);
    assert_eq!(grouped_blocks, individual_blocks);
    grouped.apply_pending();
    individual.apply_pending();
    for (a, b) in grouped.age_range.iter().zip(individual.age_range.iter()) {
        assert_eq!(a.output_counts(), b.output_counts());
        assert_eq!(a.supply_value(), b.supply_value());
        assert_eq!(
            a.realized_block_data().cap_raw,
            b.realized_block_data().cap_raw
        );
        assert_eq!(a.transfer_volume(), b.transfer_volume());
    }
    assert_eq!(grouped.age_range.under_1h.supply.utxo_count, 0);
}

#[test]
fn zero_value_spends_reduce_supply_and_increment_spent_counts() {
    let mut states = UTXOStates::new();
    states.reset().unwrap();
    let price = Cents::new(10_000);
    let timestamp = Timestamp::new(1_700_000_000);
    let supply = SupplyState {
        value: Sats::ZERO,
        utxo_count: 3,
    };
    states.receive_origins(supply, Height::ZERO, timestamp, price);
    let mut peak = PriceRangeMax::default();
    peak.extend(&[price]);
    let ctx = ComputeContext {
        starting_height: Height::ZERO,
        last_height: Height::ZERO,
        height_to_timestamp: &[timestamp],
        height_to_price: &[price],
        price_range_max: &peak,
    };
    states.send_origins([(Height::ZERO, supply)], Height::ZERO, &ctx);
    states.apply_pending();
    for counts in [
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
        assert_eq!(counts, 3);
    }
    assert_eq!(states.age_range.under_1h.supply.utxo_count, 0);
}

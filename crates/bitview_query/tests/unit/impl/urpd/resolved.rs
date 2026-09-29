use super::*;
use bitcoin::Amount;
use brk_types::{Cents, CentsCompact, Cohort, Date, Height, Sats, UrpdAggregation, UrpdWeight};
fn captured(entries: &[(u32, u64)], close: Cents) -> ResolvedUrpd {
    ResolvedUrpd {
        cohort: Cohort::new("all").unwrap(),
        height: Height::new(800_000),
        date: Date::new(2026, 9, 1),
        weight: UrpdWeight::Raw,
        aggregation: UrpdAggregation::Raw,
        close,
        entries: entries
            .iter()
            .map(|&(p, s)| (CentsCompact::new(p), Sats::new(s)))
            .collect(),
    }
}
#[test]
fn block_identity_and_supply_survive_building() {
    let input = captured(&[(100, 3), (200, 1)], Cents::new(300));
    assert_eq!(input.entries().len(), 2);
    let response = input.build().unwrap();
    assert_eq!(response.height, Height::new(800_000));
    assert_eq!(response.date, Date::new(2026, 9, 1));
    assert_eq!(response.buckets.len(), 2);
    assert!(
        captured(&[], Cents::ZERO)
            .build()
            .unwrap()
            .buckets
            .is_empty()
    );
}
#[test]
fn invalid_amounts_prices_ordering_and_market_values_are_rejected() {
    for entry in [
        (CentsCompact::NAN, Sats::new(1)),
        (CentsCompact::ZERO, Sats::MAX),
    ] {
        let mut input = captured(&[], Cents::ZERO);
        input.entries = vec![entry].into_boxed_slice();
        assert!(input.build().is_err());
    }
    for entries in [
        &[(100, Amount::MAX_MONEY.to_sat() + 1)][..],
        &[(100, 1), (100, 1)],
        &[(200, 1), (100, 1)],
    ] {
        assert!(captured(entries, Cents::ZERO).build().is_err());
    }
    for close in [Cents::NAN, Cents::new(u64::MAX - 1)] {
        assert!(
            captured(&[(100, Amount::MAX_MONEY.to_sat())], close)
                .build()
                .is_err()
        );
    }
}

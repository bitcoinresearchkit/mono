use std::fs;

use bitview_plugin_distribution::UTXOStates;
use brk_types::SupplyState;
use tempfile::tempdir;

use super::*;
use crate::{DayUrpds, ModeWeights};

#[test]
fn cutoffs_are_exclusive_and_only_occupied_buckets_count() {
    let mut bounds = AgeCutoffs::default();
    for (age, price, sats) in [
        (AgeRangeId::Under1H, 200, 1),
        (AgeRangeId::From3MTo4M, 100, 1),
        (AgeRangeId::From4MTo5M, 50, 1),
        (AgeRangeId::From4MTo5M, 300, 1),
        (AgeRangeId::From5MTo6M, 25, 1),
        (AgeRangeId::From5MTo6M, 400, 1),
        (AgeRangeId::From6MTo9M, 1, 1),
        (AgeRangeId::From6MTo9M, 900, 1),
        (AgeRangeId::Under1H, 0, 0),
        (AgeRangeId::Under1H, 1_000, 0),
    ] {
        bounds.include(age, CentsCompact::new(price), Sats::from(sats as u64));
    }
    for (range, (min, max)) in bounds.iter().zip([(100, 200), (50, 300), (25, 400)]) {
        assert_eq!(range.min, Cents::new(min));
        assert_eq!(range.max, Cents::new(max));
    }
    bounds.include(AgeRangeId::Under1H, CentsCompact::new(0), Sats::from(1_u64));
    assert_eq!(bounds.under_4m.min, Cents::ZERO);
}

#[test]
fn saved_and_live_urpds_agree_and_rewrites_replace_extremes() {
    let root = tempdir().unwrap();
    let date = Date::new(2026, 9, 14);
    let mut utxos = UTXOStates::new(root.path());
    utxos.reset().unwrap();
    for (age, price) in [
        (AgeRangeId::Under1H, 12345),
        (AgeRangeId::From3MTo4M, 45678),
        (AgeRangeId::From4MTo5M, 56789),
        (AgeRangeId::From5MTo6M, 9876),
        (AgeRangeId::From6MTo9M, 1),
    ] {
        age.select_mut(&mut utxos.age_range).receive_utxo(
            &SupplyState {
                value: Sats::from(100_u64),
                ..Default::default()
            },
            Cents::new(price),
        );
    }
    utxos.apply_pending();
    let weights = ModeWeights::from_fn(|_| None);
    utxos.write_urpds(date, root.path()).unwrap();
    let live = DayUrpds::current(&utxos, &weights);
    let saved = DayUrpds::read_if_exists(root.path(), date, &weights)
        .unwrap()
        .unwrap();
    let backfill = AgeCutoffs::read_if_exists(root.path(), date).unwrap();
    for ((live, saved), backfill) in live
        .age_price_bounds
        .iter()
        .zip(saved.age_price_bounds.iter())
        .zip(backfill.iter())
    {
        assert!(!live.min.is_nan());
        assert_eq!(live.min, saved.min);
        assert_eq!(live.max, saved.max);
        assert_eq!(live.min, backfill.min);
        assert_eq!(live.max, backfill.max);
    }
    utxos.age_range.under_1h.receive_utxo(
        &SupplyState {
            value: Sats::from(1_u64),
            ..Default::default()
        },
        Cents::ZERO,
    );
    utxos.apply_pending();
    utxos.write_urpds(date, root.path()).unwrap();
    assert_eq!(
        AgeCutoffs::read_if_exists(root.path(), date)
            .unwrap()
            .under_4m
            .min,
        Cents::ZERO
    );
}

#[test]
fn missing_and_empty_are_undefined_but_corrupt_snapshots_are_errors() {
    let root = tempdir().unwrap();
    let date = Date::new(2026, 9, 14);
    for range in AgeCutoffs::read_if_exists(root.path(), date)
        .unwrap()
        .iter()
    {
        assert!(range.min.is_nan() && range.max.is_nan());
    }
    let mut utxos = UTXOStates::new(root.path());
    utxos.reset().unwrap();
    utxos.write_urpds(date, root.path()).unwrap();
    for range in AgeCutoffs::read_if_exists(root.path(), date)
        .unwrap()
        .iter()
    {
        assert!(range.min.is_nan() && range.max.is_nan());
    }
    fs::write(AgeRangeUrpds::path(root.path(), date), b"broken").unwrap();
    assert!(AgeCutoffs::read_if_exists(root.path(), date).is_err());
}

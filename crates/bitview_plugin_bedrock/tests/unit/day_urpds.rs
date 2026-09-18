use std::iter;

use bitview_cohort::{AgeRange, AgeRangeId, UTXO_ALL_NAME};
use bitview_plugin_distribution::{AgeRangeUrpds, UTXOStates};
use brk_types::{Cents, CentsCompact, Date, PercentileId, Sats, UrpdRaw};
use tempfile::tempdir;

use super::{DayUrpds, ModeWeights};

#[test]
fn capitalized_prices_use_each_weighted_distribution_and_backfill_identically() {
    let root = tempdir().unwrap();
    let date = Date::new(2026, 9, 7);
    let names = DayUrpds::names();
    let mut weights = ModeWeights::from_fn(|_| None);
    weights.cointime = Some(AgeRange::from_fn(|age| {
        if age == AgeRangeId::Under1H { 1.0 } else { 0.5 }
    }));
    weights.coinflow = Some(AgeRange::from_fn(|age| {
        if age == AgeRangeId::Under1H { 0.5 } else { 1.0 }
    }));
    let entries = [
        (AgeRangeId::Under1H, 100),
        (AgeRangeId::Under1H, 200),
        (AgeRangeId::From5MTo6M, 400),
        (AgeRangeId::From5MTo6M, 800),
    ];
    let urpds = DayUrpds::from_age_entries(
        entries.map(|(age, p)| (age, CentsCompact::new(p), Sats::from(10_u64))),
        &weights,
    );
    let actual = urpds.capitalized_prices();
    assert_eq!(actual.all.cointime, Cents::new(500));
    assert_eq!(actual.all.coinflow, Cents::new(611));
    assert_eq!(actual.sth.cointime, Cents::new(166));
    assert_eq!(actual.sth.coinflow, Cents::new(166));
    assert_eq!(actual.lth.cointime, Cents::new(666));
    assert_eq!(actual.lth.coinflow, Cents::new(666));
    urpds.write(root.path(), &names, date).unwrap();
    let saved = DayUrpds::read_capitalized_prices(root.path(), &names, date).unwrap();
    for (a, b) in actual.iter().zip(saved.iter()) {
        assert_eq!(a.cointime, b.cointime);
        assert_eq!(a.coinflow, b.coinflow);
    }
    // A rewritten daily snapshot must not retain the old derived prices.
    DayUrpds::repeated([(300, 10)])
        .write(root.path(), &names, date)
        .unwrap();
    let rewritten = DayUrpds::read_capitalized_prices(root.path(), &names, date).unwrap();
    assert!(
        rewritten
            .iter()
            .all(|pair| pair.cointime == Cents::new(300) && pair.coinflow == Cents::new(300))
    );
}

#[test]
fn capitalized_backfill_distinguishes_missing_and_incomplete_snapshots() {
    let root = tempdir().unwrap();
    let names = DayUrpds::names();
    let date = Date::new(2026, 9, 7);
    let missing = DayUrpds::read_capitalized_prices(root.path(), &names, date).unwrap();
    assert!(
        missing
            .iter()
            .all(|pair| pair.cointime.is_nan() && pair.coinflow.is_nan())
    );
    UrpdRaw::write(
        root.path(),
        &names.all.cointime,
        date,
        [(CentsCompact::new(100), Sats::from(1_u64))].into_iter(),
    )
    .unwrap();
    assert!(DayUrpds::read_capitalized_prices(root.path(), &names, date).is_err());
}

#[test]
fn weighted_sats_are_floored_after_summing() {
    assert_eq!(DayUrpds::floor_sats(0.6 + 0.6), Sats::from(1_u64));
    assert_eq!(DayUrpds::floor_sats(0.6), Sats::ZERO);
}

#[test]
fn current_entries_build_raw_and_weighted_urpds() {
    let weights = ModeWeights::from_fn(|_| Some(AgeRange::from_fn(|_| 0.5)));
    let price = CentsCompact::new(100);
    let urpds = DayUrpds::from_age_entries(
        [
            (AgeRangeId::Under1H, price, Sats::from(3_u64)),
            (AgeRangeId::From5MTo6M, price, Sats::from(5_u64)),
        ],
        &weights,
    );

    assert_eq!(urpds.raw.map[&price], Sats::from(8_u64));
    assert_eq!(urpds.all.cointime.map[&price], Sats::from(4_u64));
    assert_eq!(urpds.age.under_5m.cointime.map[&price], Sats::from(1_u64));
    assert_eq!(urpds.long.cointime.map[&price], Sats::from(2_u64));
}

#[test]
fn names_cover_only_stored_aggregate_weights() {
    let names = DayUrpds::names();
    assert_eq!(names.all.cointime, "bedrock_cointime");
    assert_eq!(names.all.coinflow, "bedrock_coinflow");
    assert_eq!(names.sth.cointime, "bedrock_cointime_sth");
    assert_eq!(names.lth.coinflow, "bedrock_coinflow_lth");
}

#[test]
fn persisted_all_cost_basis_percentiles_match_in_memory_percentiles() {
    let root = tempdir().unwrap();
    let date = Date::new(2026, 8, 28);
    let names = DayUrpds::names();
    let urpds = DayUrpds::repeated([(100, 5), (200, 5)]);
    assert!(
        DayUrpds::read_cost_basis_if_exists(root.path(), &names, date, Cents::new(200))
            .unwrap()
            .is_none()
    );
    UrpdRaw::write(
        root.path(),
        &names.all.cointime,
        date,
        iter::once((CentsCompact::new(100), Sats::from(1_u64))),
    )
    .unwrap();
    assert!(
        DayUrpds::read_cost_basis_if_exists(root.path(), &names, date, Cents::new(200)).is_err()
    );
    urpds.write(root.path(), &names, date).unwrap();

    let expected = urpds.cost_basis(Cents::new(200));
    let actual = DayUrpds::read_cost_basis_if_exists(root.path(), &names, date, Cents::new(200))
        .unwrap()
        .expect("persisted pair");

    assert_eq!(actual.all.cointime, expected.all.cointime);
    assert_eq!(actual.all.coinflow, expected.all.coinflow);
    assert_eq!(
        actual.all.cointime.prices.per_coin[PercentileId::Pct60 as usize],
        Cents::new(200)
    );
    assert_eq!(
        actual.all.cointime.prices.per_dollar[PercentileId::Pct50 as usize],
        Cents::new(200)
    );
}

#[test]
fn historical_read_uses_packed_source_without_legacy_all_file() {
    let root = tempdir().unwrap();
    let date = Date::new(2026, 8, 26);
    let mut utxos = UTXOStates::new(root.path());
    utxos.reset().unwrap();
    utxos.write_urpds(date, root.path()).unwrap();

    assert!(AgeRangeUrpds::path(root.path(), date).exists());
    assert!(!UrpdRaw::path(root.path(), UTXO_ALL_NAME.id, date).exists());

    let weights = ModeWeights::from_fn(|_| None);
    let urpds = DayUrpds::read_if_exists(root.path(), date, &weights)
        .unwrap()
        .expect("packed source");
    assert!(urpds.raw.map.is_empty());
}

#[test]
fn density_uses_mode_weights_and_backfills_at_the_represented_spot() {
    let root = tempdir().unwrap();
    let date = Date::new(2026, 9, 12);
    let names = DayUrpds::names();
    let mut weights = ModeWeights::from_fn(|_| None);
    weights.cointime = Some(AgeRange::from_fn(|age| {
        if age == AgeRangeId::Under1H { 1.0 } else { 0.5 }
    }));
    weights.coinflow = Some(AgeRange::from_fn(|age| {
        if age == AgeRangeId::Under1H { 0.5 } else { 1.0 }
    }));
    let urpds = DayUrpds::from_age_entries(
        [
            (AgeRangeId::Under1H, 95),
            (AgeRangeId::From5MTo6M, 105),
            (AgeRangeId::From5MTo6M, 200),
        ]
        .map(|(age, price)| (age, CentsCompact::new(price), Sats::from(10_u64))),
        &weights,
    );
    let spot = Cents::new(100);
    let actual = urpds.cost_basis(spot);
    let cointime = &actual.all.cointime.supply_density;
    let coinflow = &actual.all.coinflow.supply_density;
    assert!((f64::from(cointime.total) - 0.75).abs() < 1e-9);
    assert!((f64::from(cointime.in_profit) - 0.5).abs() < 1e-9);
    assert!((f64::from(cointime.in_loss) - 0.25).abs() < 1e-9);
    assert!((f64::from(coinflow.total) - 0.6).abs() < 1e-9);
    assert!((f64::from(coinflow.in_profit) - 0.2).abs() < 1e-9);
    assert!((f64::from(coinflow.in_loss) - 0.4).abs() < 1e-9);
    urpds.write(root.path(), &names, date).unwrap();
    let saved = DayUrpds::read_cost_basis_if_exists(root.path(), &names, date, spot)
        .unwrap()
        .unwrap();
    assert_eq!(actual.all.cointime, saved.all.cointime);
    assert_eq!(actual.all.coinflow, saved.all.coinflow);
    let repriced = DayUrpds::read_cost_basis_if_exists(root.path(), &names, date, Cents::new(200))
        .unwrap()
        .unwrap();
    assert!((f64::from(repriced.all.cointime.supply_density.total) - 0.25).abs() < 1e-9);
    assert_eq!(repriced.all.cointime.prices, actual.all.cointime.prices);

    DayUrpds::repeated([(100, 10)])
        .write(root.path(), &names, date)
        .unwrap();
    let rewritten = DayUrpds::read_cost_basis_if_exists(root.path(), &names, date, spot)
        .unwrap()
        .unwrap();
    assert_eq!(
        f64::from(rewritten.all.cointime.supply_density.in_profit),
        1.0
    );
    assert_eq!(
        f64::from(rewritten.all.coinflow.supply_density.in_loss),
        0.0
    );
}

#[test]
fn ten_percent_density_has_inclusive_boundaries_and_matches_saved_snapshots() {
    let root = tempdir().unwrap();
    let date = Date::new(2026, 9, 12);
    let names = DayUrpds::names();
    let urpds = DayUrpds::repeated([
        (89, 10),
        (90, 10),
        (94, 10),
        (95, 10),
        (100, 20),
        (105, 10),
        (106, 10),
        (110, 10),
        (111, 10),
    ]);
    let data = urpds.cost_basis(Cents::new(100));
    for mode in data.all.iter() {
        assert_eq!(mode.supply_density.total.inner(), 400_000);
        assert_eq!(mode.supply_density_10pct.total.inner(), 800_000);
        assert_eq!(mode.supply_density_10pct.in_profit.inner(), 500_000);
        assert_eq!(mode.supply_density_10pct.in_loss.inner(), 300_000);
    }
    urpds.write(root.path(), &names, date).unwrap();
    let saved = DayUrpds::read_cost_basis_if_exists(root.path(), &names, date, Cents::new(100))
        .unwrap()
        .unwrap();
    assert_eq!(saved.all.cointime, data.all.cointime);
    assert_eq!(saved.all.coinflow, data.all.coinflow);
    // Fractional lower boundary: 90% of 101 is 90.9, so the 90-cent bucket is excluded.
    let fractional =
        DayUrpds::repeated([(90, 10), (91, 10), (111, 10), (112, 10)]).cost_basis(Cents::new(101));
    assert_eq!(
        fractional.all.cointime.supply_density_10pct.total.inner(),
        500_000
    );
}

#[test]
fn final_maps_floor_combined_mass_and_omit_zero_buckets_in_every_mode() {
    let weights = ModeWeights::from_fn(|_| Some(AgeRange::from_fn(|_| 0.6)));
    let entries = [
        (AgeRangeId::Under1H, 200, 1),
        (AgeRangeId::From5MTo6M, 100, 1),
        (AgeRangeId::Under1H, 100, 1_u64),
        (AgeRangeId::From5MTo6M, 200, 0),
    ]
    .map(|(age, price, sats)| (age, CentsCompact::new(price), Sats::from(sats as u64)));
    let urpds = DayUrpds::from_age_entries(entries, &weights);
    for mode in urpds.all.iter() {
        assert_eq!(mode.map.len(), 1);
        assert_eq!(mode.map[&CentsCompact::new(100)], Sats::_1);
    }
    for mode in urpds.age.under_5m.iter().chain(urpds.long.iter()) {
        assert!(mode.map.is_empty());
    }
    assert_eq!(urpds.raw.map[&CentsCompact::new(100)], Sats::from(2_u64));
    assert_eq!(urpds.raw.map[&CentsCompact::new(200)], Sats::_1);

    let unavailable = DayUrpds::from_age_entries(entries, &ModeWeights::from_fn(|_| None));
    assert_eq!(unavailable.raw.map, urpds.raw.map);
    assert!(unavailable.all.iter().all(|mode| mode.map.is_empty()));
    let empty = DayUrpds::from_age_entries([], &weights);
    assert!(empty.raw.map.is_empty());
    assert!(empty.all.iter().all(|mode| mode.map.is_empty()));
}

#[test]
fn age_densities_use_exclusive_cutoffs_and_each_cohorts_weighted_denominator() {
    let mut weights = ModeWeights::from_fn(|_| None);
    weights.cointime = Some(AgeRange::from_fn(|age| {
        if age == AgeRangeId::Under1H { 1.0 } else { 0.5 }
    }));
    weights.coinflow = Some(AgeRange::from_fn(|age| {
        if age == AgeRangeId::Under1H { 0.5 } else { 1.0 }
    }));
    let urpds = DayUrpds::from_age_entries(
        [
            (AgeRangeId::Under1H, 100, 20_u64),
            (AgeRangeId::From3MTo4M, 110, 20),
            (AgeRangeId::From4MTo5M, 105, 20),
            (AgeRangeId::From5MTo6M, 200, 20),
            (AgeRangeId::From6MTo9M, 100, 1000),
        ]
        .map(|(age, price, sats)| (age, CentsCompact::new(price), Sats::from(sats))),
        &weights,
    );
    let densities = urpds.age_densities(Cents::new(100));
    for (actual, expected) in densities.iter().zip([
        [(2.0 / 3.0, 1.0), (1.0 / 3.0, 1.0)],
        [(0.75, 1.0), (0.6, 1.0)],
        [(0.6, 0.8), (3.0 / 7.0, 5.0 / 7.0)],
    ]) {
        for (mode, (five, ten)) in actual.iter().zip(expected) {
            assert!((f64::from(mode.supply_density.total) - five).abs() <= 1e-6);
            assert!((f64::from(mode.supply_density_10pct.total) - ten).abs() <= 1e-6);
            for band in mode.iter() {
                assert!(
                    (f64::from(band.total) - f64::from(band.in_profit) - f64::from(band.in_loss))
                        .abs()
                        <= 1e-6
                );
            }
        }
    }
    // The standard STH cohort is exactly the <5m distribution, not a second aggregate.
    assert_eq!(
        urpds
            .age
            .under_5m
            .cointime
            .map
            .values()
            .map(|s| u64::from(*s))
            .sum::<u64>(),
        40
    );
}

#[test]
fn age_density_floors_after_combining_ages_and_missing_weights_stay_undefined() {
    let mut weights = ModeWeights::from_fn(|_| None);
    weights.cointime = Some(AgeRange::from_fn(|_| 0.6));
    let urpds = DayUrpds::from_age_entries(
        [
            (AgeRangeId::Under1H, 100, 1_u64),
            (AgeRangeId::From3MTo4M, 100, 1),
            (AgeRangeId::From4MTo5M, 200, 2),
        ]
        .map(|(age, price, sats)| (age, CentsCompact::new(price), Sats::from(sats))),
        &weights,
    );
    let densities = urpds.age_densities(Cents::new(100));
    assert_eq!(
        densities.under_4m.cointime.supply_density.total.inner(),
        1_000_000
    );
    assert_eq!(
        densities.under_5m.cointime.supply_density.total.inner(),
        500_000
    );
    for cohort in densities.iter() {
        assert!(cohort.coinflow.supply_density.total.is_nan());
    }
    for cohort in urpds.age_densities(Cents::ZERO).iter() {
        assert!(cohort.cointime.supply_density.total.is_nan());
    }
}

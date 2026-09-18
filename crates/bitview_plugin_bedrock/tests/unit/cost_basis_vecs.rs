use std::{collections::BTreeSet, fs, sync::Once};

use bitview_cohort::{AgeRange, AgeRangeId, UTXOAggregate};
use bitview_plugin_distribution::UTXOStates;
use bitview_traversable::Traversable;
use bitview_vecs::DailyMappings;
use brk_exit::Exit;
use brk_types::{Cents, CentsCompact, Date, Day1, Height, Sats, SupplyState, UrpdRaw, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Budgeted, Database, EagerVec, ImportableVec, LazyVec, PcoVec, PcoVecValue,
    ReadableCloneableVec, ReadableVec, VecIndex, WritableVec,
};

use crate::{
    AgeCutoffs, AgeDensityVecs, AgePriceBoundsVecs, CostBasisVecs, DayUrpds, ModeWeights,
    WeightedPair,
};

fn mapping<I: VecIndex, T: PcoVecValue>(db: &Database, name: &str) -> LazyVec<I, Day1, I, T> {
    let source: EagerVec<PcoVec<I, T>> = EagerVec::forced_import(db, name, Version::ONE).unwrap();
    LazyVec::init(
        name,
        Version::ONE,
        source.read_only_boxed_clone(),
        |_, _| Day1::from(0_usize),
    )
}

fn daily_mappings(db: &Database) -> DailyMappings {
    static CACHE: Once = Once::new();
    CACHE.call_once(|| {
        Budgeted::init_global(64 * 1024 * 1024).unwrap();
    });
    let mut heights: EagerVec<PcoVec<Height, Day1>> =
        EagerVec::forced_import(db, "height_day1", Version::ONE).unwrap();
    heights.push(Day1::from(0_usize));
    heights.push(Day1::from(1_usize));
    heights.write().unwrap();
    DailyMappings {
        height: heights.read_only_boxed_clone(),
        minute10: mapping(db, "minute10"),
        minute30: mapping(db, "minute30"),
        hour1: mapping(db, "hour1"),
        hour4: mapping(db, "hour4"),
        hour12: mapping(db, "hour12"),
        day3: mapping(db, "day3"),
        week1: mapping(db, "week1"),
        month1: mapping(db, "month1"),
        month3: mapping(db, "month3"),
        month6: mapping(db, "month6"),
        year1: mapping(db, "year1"),
        year10: mapping(db, "year10"),
        halving: mapping(db, "halving"),
        epoch: mapping(db, "epoch"),
    }
}

#[test]
fn density_series_persist_reopen_rewind_and_expose_normal_percent_units() {
    let root = tempdir().unwrap();
    let db = Database::open(root.path()).unwrap();
    let mappings = daily_mappings(&db);
    let import = || CostBasisVecs::forced_import(&db, Version::ONE, &mappings).unwrap();
    let mut vecs = import();
    let data = DayUrpds::repeated([(95, 20), (100, 30), (105, 10), (110, 20), (200, 20)])
        .cost_basis(Cents::new(100));
    vecs.push(&data);
    vecs.push(&UTXOAggregate::default());
    assert_eq!(vecs.minimum_len(), 2);
    let names = vecs
        .supply_density
        .iter_any_visible()
        .chain(vecs.supply_density_10pct.iter_any_visible())
        .map(|vec| vec.name())
        .collect::<BTreeSet<_>>();
    for mode in ["cointime", "coinflow"] {
        for side in ["", "_in_profit", "_in_loss"] {
            for unit in ["", "_ratio", "_ppm"] {
                for band in ["", "_10pct"] {
                    assert!(names.contains(
                        format!("bedrock_{mode}_supply_density{band}{side}{unit}").as_str()
                    ));
                }
            }
        }
    }
    for vec in vecs.stored_vecs_mut() {
        vec.write().unwrap();
    }
    drop(vecs);
    let mut vecs = import();
    for mode in vecs.supply_density_10pct.iter() {
        assert_eq!(
            mode.series
                .total
                .ppm
                .day1
                .collect_one(Day1::from(0_usize))
                .unwrap()
                .inner(),
            800_000
        );
        assert!(
            mode.series
                .total
                .ppm
                .day1
                .collect_one(Day1::from(1_usize))
                .unwrap()
                .is_nan()
        );
    }
    for mode in vecs.supply_density.iter() {
        let total = &mode.series.total;
        assert_eq!(
            total
                .ppm
                .day1
                .collect_one(Day1::from(0_usize))
                .unwrap()
                .inner(),
            600_000
        );
        assert!(
            (f32::from(total.percent.day1.collect_one(Day1::from(0_usize)).unwrap()) - 60.0).abs()
                < 1e-5
        );
        assert!(
            (f32::from(total.ratio.day1.collect_one(Day1::from(0_usize)).unwrap()) - 0.6).abs()
                < 1e-6
        );
        assert!(
            total
                .ppm
                .day1
                .collect_one(Day1::from(1_usize))
                .unwrap()
                .is_nan()
        );
        assert!(
            total
                .ppm
                .views
                .height
                .collect_one(Height::from(1_usize))
                .unwrap()
                .unwrap()
                .is_nan()
        );
    }
    for vec in vecs.stored_vecs_mut() {
        vec.any_truncate_if_needed_at(1).unwrap();
    }
    let changed = DayUrpds::repeated([(105, 10)]).cost_basis(Cents::new(100));
    vecs.push(&changed);
    assert_eq!(vecs.minimum_len(), 2);
    for vec in vecs.stored_vecs_mut() {
        vec.write().unwrap();
    }
    for mode in vecs
        .supply_density
        .iter()
        .chain(vecs.supply_density_10pct.iter())
    {
        assert_eq!(
            mode.series
                .in_loss
                .ppm
                .day1
                .collect_one(Day1::from(1_usize))
                .unwrap()
                .inner(),
            1_000_000
        );
        assert_eq!(
            mode.series
                .in_profit
                .ppm
                .day1
                .collect_one(Day1::from(1_usize))
                .unwrap()
                .inner(),
            0
        );
    }
}

#[test]
fn age_bounds_backfill_persist_rewind_and_reset_together() {
    let root = tempdir().unwrap();
    let db = Database::open(&root.path().join("vecs")).unwrap();
    let mappings = daily_mappings(&db);
    let mut dates: EagerVec<PcoVec<Day1, Date>> =
        EagerVec::forced_import(&db, "dates", Version::ONE).unwrap();
    let first = Date::new(2026, 9, 12);
    dates.push(first);
    dates.push(Date::new(2026, 9, 13));
    dates.write().unwrap();
    let mut utxos = UTXOStates::new(root.path());
    utxos.reset().unwrap();
    utxos.age_range.under_1h.receive_utxo(
        &SupplyState {
            value: Sats::from(1_u64),
            ..Default::default()
        },
        Cents::new(12345),
    );
    utxos.apply_pending();
    utxos.write_urpds(first, root.path()).unwrap();
    let expected = AgeCutoffs::read_if_exists(root.path(), first).unwrap();
    let import = || AgePriceBoundsVecs::forced_import(&db, Version::ONE, &mappings).unwrap();
    let mut vecs = import();
    assert_eq!(vecs.prepare(Version::ONE, 2, 2).unwrap(), 0);
    vecs.backfill(&dates, root.path(), 0, 2, &Exit::default())
        .unwrap();
    drop(vecs);
    let mut vecs = import();
    for (view, expected) in vecs.series.iter().zip(expected.iter()) {
        assert_eq!(
            view.min
                .cents
                .day1
                .collect_one(Day1::from(0_usize))
                .unwrap(),
            expected.min
        );
        assert_eq!(
            view.max
                .cents
                .views
                .height
                .collect_one(Height::ZERO)
                .flatten()
                .unwrap(),
            expected.max
        );
        assert!(
            view.min
                .cents
                .day1
                .collect_one(Day1::from(1_usize))
                .unwrap()
                .is_nan()
        );
    }
    assert_eq!(vecs.prepare(Version::ONE, 1, 2).unwrap(), 1);
    let mut updated = AgeCutoffs::default();
    updated.include(
        AgeRangeId::Under1H,
        CentsCompact::new(700),
        Sats::from(1_u64),
    );
    vecs.push(&updated);
    for vec in vecs.stored_vecs_mut() {
        vec.write().unwrap();
    }
    for view in vecs.series.iter() {
        assert_eq!(
            view.max
                .cents
                .day1
                .collect_one(Day1::from(1_usize))
                .unwrap(),
            Cents::new(700)
        );
    }
    // A missing series rewinds all six, even if the date source is ahead.
    vecs.stored.under_5m.min.truncate_if_needed_at(0).unwrap();
    assert_eq!(vecs.prepare(Version::ONE, 2, 2).unwrap(), 0);
    vecs.push(&updated);
    assert_eq!(vecs.prepare(Version::TWO, 2, 2).unwrap(), 0);
    for vec in vecs.stored_vecs_mut() {
        assert_eq!(vec.len(), 0);
    }
}

#[test]
fn age_density_live_and_saved_inputs_persist_rewind_and_reset_together() {
    let root = tempdir().unwrap();
    let db = Database::open(&root.path().join("vecs")).unwrap();
    let mappings = daily_mappings(&db);
    let date = Date::new(2026, 9, 16);
    let mut utxos = UTXOStates::new(root.path());
    utxos.reset().unwrap();
    for (age, price, sats) in [
        (AgeRangeId::Under1H, 10000, 30_u64),
        (AgeRangeId::From3MTo4M, 11000, 20),
        (AgeRangeId::From4MTo5M, 10500, 20),
        (AgeRangeId::From5MTo6M, 20000, 40),
        (AgeRangeId::From6MTo9M, 10000, 100),
    ] {
        age.select_mut(&mut utxos.age_range).receive_utxo(
            &SupplyState {
                value: Sats::from(sats),
                ..Default::default()
            },
            Cents::new(price),
        );
    }
    utxos.apply_pending();
    utxos.write_urpds(date, root.path()).unwrap();
    let weights = ModeWeights::from_fn(|_| Some(AgeRange::from_fn(|_| 0.6)));
    let expected = DayUrpds::current(&utxos, &weights).age_densities(Cents::new(10000));
    let saved = DayUrpds::read_if_exists(root.path(), date, &weights)
        .unwrap()
        .unwrap()
        .age_densities(Cents::new(10000));
    for (live, saved) in expected.iter().zip(saved.iter()) {
        assert_eq!(live.cointime, saved.cointime);
        assert_eq!(live.coinflow, saved.coinflow);
    }
    let import = || AgeDensityVecs::forced_import(&db, Version::ONE, &mappings).unwrap();
    let mut vecs = import();
    assert_eq!(vecs.prepare(Version::ONE, 2, 2).unwrap(), 0);
    vecs.push(&saved);
    vecs.push(&AgeCutoffs::default());
    for vec in vecs.stored_vecs_mut() {
        vec.write().unwrap();
    }
    drop(vecs);
    let mut vecs = import();
    let names = vecs
        .iter_any_visible()
        .map(|v| v.name())
        .collect::<BTreeSet<_>>();
    for age in ["under_4m", "under_5m", "under_6m"] {
        for mode in ["cointime", "coinflow"] {
            for band in ["", "_10pct"] {
                for side in ["", "_in_profit", "_in_loss"] {
                    for unit in ["", "_ratio", "_ppm"] {
                        assert!(
                            names.contains(
                                format!("bedrock_{mode}_{age}_supply_density{band}{side}{unit}")
                                    .as_str()
                            )
                        );
                    }
                }
            }
        }
    }
    let views = vecs
        .series
        .iter()
        .flat_map(WeightedPair::iter)
        .flat_map(|bands| bands.iter());
    let values = expected
        .iter()
        .flat_map(WeightedPair::iter)
        .flat_map(|bands| bands.iter());
    for (view, expected) in views.zip(values) {
        assert_eq!(
            view.series
                .total
                .ppm
                .day1
                .collect_one(Day1::from(0_usize))
                .unwrap(),
            expected.total
        );
        assert!(
            view.series
                .total
                .ppm
                .day1
                .collect_one(Day1::from(1_usize))
                .unwrap()
                .is_nan()
        );
    }
    assert_eq!(vecs.prepare(Version::ONE, 1, 2).unwrap(), 1);
    vecs.push(&expected);
    for vec in vecs.stored_vecs_mut() {
        assert_eq!(vec.len(), 2);
    }
    assert_eq!(vecs.prepare(Version::TWO, 2, 2).unwrap(), 0);
    for vec in vecs.stored_vecs_mut() {
        assert_eq!(vec.len(), 0);
    }
}

#[test]
fn term_distributions_keep_cohort_and_weighting_separate_through_backfill_and_storage() {
    let root = tempdir().unwrap();
    let db = Database::open(&root.path().join("vecs")).unwrap();
    let mappings = daily_mappings(&db);
    let date = Date::new(2026, 9, 17);
    let names = DayUrpds::names();
    let mut utxos = UTXOStates::new(root.path());
    utxos.reset().unwrap();
    for (age, price) in [
        (AgeRangeId::Under1H, 100),
        (AgeRangeId::From3MTo4M, 200),
        (AgeRangeId::From5MTo6M, 400),
        (AgeRangeId::From6MTo9M, 800),
    ] {
        age.select_mut(&mut utxos.age_range).receive_utxo(
            &SupplyState {
                value: Sats::from(10_u64),
                ..Default::default()
            },
            Cents::new(price),
        );
    }
    utxos.apply_pending();
    let mut weights = ModeWeights::from_fn(|_| None);
    weights.cointime = Some(AgeRange::from_fn(|_| 1.0));
    weights.coinflow = Some(AgeRange::from_fn(|age| {
        if matches!(age, AgeRangeId::Under1H | AgeRangeId::From5MTo6M) {
            0.1
        } else {
            1.0
        }
    }));
    let urpds = DayUrpds::current(&utxos, &weights);
    let live = urpds.cost_basis(Cents::new(200));
    urpds.write(root.path(), &names, date).unwrap();
    let saved = DayUrpds::read_cost_basis_if_exists(root.path(), &names, date, Cents::new(200))
        .unwrap()
        .unwrap();
    for (live, saved) in live.iter().zip(saved.iter()) {
        assert_eq!(live.cointime, saved.cointime);
        assert_eq!(live.coinflow, saved.coinflow);
    }
    let import = || CostBasisVecs::forced_import(&db, Version::ONE, &mappings).unwrap();
    let mut vecs = import();
    assert_eq!(vecs.minimum_len(), 0);
    vecs.push(&saved);
    vecs.push(&UTXOAggregate::default());
    for vec in vecs.stored_vecs_mut() {
        vec.write().unwrap();
    }
    drop(vecs);
    let mut vecs = import();
    assert_eq!(vecs.minimum_len(), 2);
    for (cohort, name, low, high) in [(&vecs.sth, "sth", 100, 200), (&vecs.lth, "lth", 400, 800)] {
        for (weighting, pairs, expected) in [
            ("per_coin", &cohort.per_coin, [low, high]),
            ("per_dollar", &cohort.per_dollar, [high, high]),
        ] {
            for ((mode, prices), expected) in ["cointime", "coinflow"]
                .into_iter()
                .zip(pairs.iter())
                .zip(expected)
            {
                assert_eq!(
                    prices.pct50.cents.day1.collect_one(Day1::from(0_usize)),
                    Some(Cents::new(expected))
                );
                assert!(
                    prices
                        .pct50
                        .cents
                        .day1
                        .collect_one(Day1::from(1_usize))
                        .unwrap()
                        .is_nan()
                );
                assert!(prices.iter_any_visible().any(|vec| vec.name()
                    == format!("bedrock_{mode}_{name}_cost_basis_{weighting}_pct50_cents")));
            }
        }
    }
    // Missing cohort histories must trigger backfill even when all-holder data exists.
    for vec in vecs.sth.stored_vecs_mut() {
        vec.any_truncate_if_needed_at(0).unwrap();
    }
    assert_eq!(vecs.minimum_len(), 0);
    for vec in vecs.stored_vecs_mut() {
        vec.any_validate_computed_version_or_reset(Version::new(99))
            .unwrap();
        assert_eq!(vec.len(), 0);
    }
    fs::remove_file(UrpdRaw::path(root.path(), &names.sth.coinflow, date)).unwrap();
    assert!(
        DayUrpds::read_cost_basis_if_exists(root.path(), &names, date, Cents::new(200)).is_err()
    );
    fs::remove_file(UrpdRaw::path(root.path(), &names.sth.cointime, date)).unwrap();
    let missing = DayUrpds::read_cost_basis_if_exists(root.path(), &names, date, Cents::new(200))
        .unwrap()
        .unwrap();
    assert!(
        missing
            .sth
            .cointime
            .prices
            .per_coin
            .iter()
            .all(|v| v.is_nan())
    );
    assert_eq!(missing.lth.cointime, live.lth.cointime);
}

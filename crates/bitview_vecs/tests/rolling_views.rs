use bitview_collections::*;
use bitview_compute::*;
use bitview_transforms::{AvgCentsToUsd, AvgSatsToBtc, CentsUnsignedToDollars, SatsToBitcoin};
use bitview_vecs::*;
use brk_types::{
    Cents, CentsSigned, Height, PartsPerMillionSigned64, Sats, SatsSigned, Timestamp, Version,
};
use common::{indexes, stored};
use schemars::JsonSchema;
use tempfile::tempdir;
use vecdb::{AnyVec, Database, ReadableCloneableVec, ReadableVec, UnaryTransform};

mod common;

fn check_conversion<T, S, F>(
    view: &LazyPerBlock<T, S>,
    height: &impl ReadableVec<Height, S>,
    resolutions: &Resolutions<S>,
    name: &str,
    version: Version,
) where
    T: NumericValue + JsonSchema,
    S: NumericValue + JsonSchema,
    F: UnaryTransform<S, T>,
{
    assert_eq!(view.height.name(), name);
    assert_eq!(view.height.version(), version + height.version());
    assert_eq!(
        view.height.collect(),
        height
            .collect()
            .into_iter()
            .map(F::apply)
            .collect::<Vec<_>>()
    );
    macro_rules! period {
        ($($field:ident),+ $(,)?) => {$(
            assert_eq!(view.$field.name(), name);
            assert_eq!(view.$field.version(), version + resolutions.$field.version());
            assert_eq!(
                view.$field.collect(),
                resolutions.$field.collect().into_iter()
                    .map(|value| value.map(F::apply)).collect::<Vec<_>>()
            );
        )+};
    }
    period!(
        minute10, minute30, hour1, hour4, hour12, day1, day3, week1, month1, month3, month6, year1,
        year10
    );
    macro_rules! epoch {
        ($($field:ident),+ $(,)?) => {$(
            assert_eq!(view.$field.name(), name);
            assert_eq!(view.$field.version(), version + resolutions.$field.version());
            assert_eq!(
                view.$field.collect(),
                resolutions.$field.collect().into_iter().map(F::apply).collect::<Vec<_>>()
            );
        )+};
    }
    epoch!(halving, epoch);
}

#[test]
fn rolling_units_preserve_height_and_all_resolution_views() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = indexes(&db);
    macro_rules! mappings {
        ($($field:ident),+ $(,)?) => {$(
            indexes.first_height.$field = common::first_heights(concat!("populated_", stringify!($field)),
                [0usize, 2, 2, 7, 200].map(Height::from),
            );
        )+};
    }
    mappings!(
        minute10, minute30, hour1, hour4, hour12, day1, day3, week1, month1, month3, month6, year1,
        year10
    );
    indexes.first_height.halving = common::first_heights("populated_halving", [Height::ZERO]);
    indexes.first_height.epoch = common::first_heights("populated_epoch", [Height::ZERO]);

    let timestamps = stored::<Height, _>(
        &db,
        "timestamps",
        (0..32u32).map(|i| Timestamp::from(i * i * 43_200)),
    );
    let starts = Windows {
        _24h: 1,
        _1w: 7,
        _1m: 30,
        _1y: 365,
    }
    .map_with_suffix(|suffix, &days| {
        LazyWindowStartVec::days(suffix, Version::new(3), days, &timestamps)
    });
    let starts_ref = Windows {
        _24h: &starts._24h,
        _1w: &starts._1w,
        _1m: &starts._1m,
        _1y: &starts._1y,
    };
    let sats_values: Vec<_> = (0..32u64)
        .map(|i| Sats::from(i * (i + 1) * 50_000_003))
        .collect();
    let cents_values: Vec<_> = (0..32u64)
        .map(|i| Cents::from(i * (i + 1) * 12_347))
        .collect();
    let sats = stored::<Height, _>(&db, "sats", sats_values.iter().copied());
    let cents = stored::<Height, _>(&db, "cents", cents_values.iter().copied());
    let spot_sats = stored::<Height, _>(
        &db,
        "spot_sats",
        (0..32u64).map(|i| Sats::from((i % 7) * 100_000_003)),
    );
    let spot_cents = stored::<Height, _>(
        &db,
        "spot_cents",
        (0..32u64).map(|i| Cents::from((i % 5) * 10_003)),
    );
    let version = Version::new(7);
    let sums =
        LazyRollingSumsAmountFromHeight::new("sum", version, &sats, &cents, &starts_ref, &indexes);
    let averages =
        LazyRollingAvgsAmountFromHeight::new("avg", version, &sats, &cents, &starts_ref, &indexes);
    let fiat = LazyRollingSumsFiatFromHeight::new("fiat", version, &cents, &starts_ref, &indexes);
    let amount_delta =
        LazyRollingDeltasAmountFromHeight::<Sats, SatsSigned, PartsPerMillionSigned64>::new(
            "amount_delta",
            version,
            &spot_sats,
            &starts_ref,
            &indexes,
        );
    let scalar_delta =
        LazyRollingDeltasFromHeight::<Sats, SatsSigned, PartsPerMillionSigned64>::new(
            "scalar_delta",
            version,
            &spot_sats,
            &starts_ref,
            &indexes,
        );
    let fiat_delta =
        LazyRollingDeltasFiatFromHeight::<Cents, CentsSigned, PartsPerMillionSigned64>::new(
            "fiat_delta",
            version,
            &spot_cents,
            &starts_ref,
            &indexes,
        );

    for (slot, suffix) in Windows::<()>::SUFFIXES.into_iter().enumerate() {
        let sum = sums.as_array()[slot];
        let avg = averages.as_array()[slot];
        let fiat = fiat.as_array()[slot];
        let start = starts.as_array()[slot];
        let fiat_avg =
            LazyRollingAvgFiatFromHeight::new("fiat_avg", version, &cents, start, &indexes);
        let expected_sats: Vec<_> = start
            .collect()
            .into_iter()
            .enumerate()
            .map(|(i, start)| {
                sats_values[i]
                    - usize::from(start)
                        .checked_sub(1)
                        .map(|j| sats_values[j])
                        .unwrap_or_default()
            })
            .collect();
        assert_eq!(sum.sats.height.collect(), expected_sats);
        assert_eq!(
            sum.sats.resolutions.day1.collect(),
            vec![
                Some(expected_sats[1]),
                None,
                Some(expected_sats[6]),
                Some(expected_sats[31]),
                None,
            ]
        );
        check_conversion::<_, _, SatsToBitcoin>(
            &sum.btc,
            &sum.sats.height,
            &sum.sats.resolutions,
            &format!("sum_{suffix}"),
            version,
        );
        check_conversion::<_, _, CentsUnsignedToDollars>(
            &sum.usd,
            &sum.cents.height,
            &sum.cents.resolutions,
            &format!("sum_{suffix}_usd"),
            version,
        );
        check_conversion::<_, _, AvgSatsToBtc>(
            &avg.btc,
            &avg.sats.height,
            &avg.sats.resolutions,
            &format!("avg_{suffix}"),
            version,
        );
        check_conversion::<_, _, AvgCentsToUsd>(
            &avg.usd,
            &avg.cents.height,
            &avg.cents.resolutions,
            &format!("avg_{suffix}_usd"),
            version,
        );
        check_conversion::<_, _, CentsUnsignedToDollars>(
            &fiat.usd,
            &fiat.cents.height,
            &fiat.cents.resolutions,
            &format!("fiat_{suffix}"),
            version,
        );
        check_conversion::<_, _, AvgCentsToUsd>(
            &fiat_avg.usd,
            &fiat_avg.cents.height,
            &fiat_avg.cents.resolutions,
            "fiat_avg",
            version,
        );
        let amount = amount_delta.absolute.as_array()[slot];
        let money = fiat_delta.absolute.as_array()[slot];
        let scalar = scalar_delta.absolute.as_array()[slot];
        let spot = spot_sats.collect();
        let expected_delta: Vec<_> = start
            .collect()
            .into_iter()
            .enumerate()
            .map(|(i, start)| {
                SatsSigned::from(f64::from(spot[i]) - f64::from(spot[usize::from(start)]))
            })
            .collect();
        assert_eq!(scalar.height.collect(), expected_delta);
        assert_eq!(amount.sats.height.collect(), expected_delta);
        assert_eq!(scalar.height.name(), format!("scalar_delta_{suffix}"));
        assert_eq!(
            amount.sats.height.name(),
            format!("amount_delta_{suffix}_sats")
        );
        assert_eq!(
            money.cents.height.name(),
            format!("fiat_delta_{suffix}_cents")
        );
        assert_eq!(
            scalar.height.version(),
            version + spot_sats.read_only_boxed_clone().version() + start.version()
        );
        check_conversion::<_, _, <SatsSigned as AmountType>::ToBitcoin>(
            &amount.btc,
            &amount.sats.height,
            &amount.sats.resolutions,
            &format!("amount_delta_{suffix}"),
            version,
        );
        check_conversion::<_, _, <CentsSigned as FiatType>::ToDollars>(
            &money.usd,
            &money.cents.height,
            &money.cents.resolutions,
            &format!("fiat_delta_{suffix}"),
            version,
        );
        macro_rules! check_rate {
            ($name:literal, $rate:expr) => {{
                let name = $name;
                let rate = $rate;
                assert_eq!(rate.ppm.height.name(), format!("{name}_{suffix}_rate_ppm"));
                check_conversion::<_, _, <PartsPerMillionSigned64 as FixedRatio>::ToRatio>(
                    &rate.ratio,
                    &rate.ppm.height,
                    &rate.ppm.resolutions,
                    &format!("{name}_{suffix}_rate_ratio"),
                    version,
                );
                check_conversion::<_, _, <PartsPerMillionSigned64 as FixedRatio>::ToPercent>(
                    &rate.percent,
                    &rate.ppm.height,
                    &rate.ppm.resolutions,
                    &format!("{name}_{suffix}_rate"),
                    version,
                );
            }};
        }
        check_rate!("amount_delta", amount_delta.rate.as_array()[slot]);
        check_rate!("fiat_delta", fiat_delta.rate.as_array()[slot]);
        check_rate!("scalar_delta", scalar_delta.rate.as_array()[slot]);
    }
}

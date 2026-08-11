use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fs,
    io::{Error, ErrorKind},
    path::Path,
};

use brk_cohort::{
    AgeRange, AgeRangeId, ByTerm, CohortContext, TERM_FILTERS, TERM_NAMES, UTXO_ALL_NAME,
};
use brk_error::Result;
use brk_indexer::Indexer;
use brk_types::{Cents, CentsCompact, Date, Day1, Sats, StoredF64, UrpdRaw, UrpdWeight, Version};
use vecdb::{AnyStoredVec, AnyVec, ColumnId, Exit, ReadableVec, VecValue};

use super::{
    vecs::{
        LEVEL_IDS, Levels, LossPercentileId, ModeId, ModeVecs, Modes, Percentiles, PriceBandId,
        PriceBands, Vecs, WeightedModeId, WeightedModes,
    },
    weighted_urpd_name,
};
use crate::{
    distribution::{self, UTXOStates},
    frameworks::{
        self,
        coinflow::{AgeBand, HORIZON_COUNT, HorizonId, age_bounds_days, horizon_mobility},
    },
    indexes,
    internal::db_utils::validate_any_computed_version_or_reset,
};

const MIN_CALIBRATION_DAYS: usize = 365;
const WRITE_INTERVAL_DAYS: usize = 100;
const PERCENTILES: Percentiles<f64> = Percentiles {
    pct95: 0.95,
    pct98: 0.98,
    pct99: 0.99,
    pct99_5: 0.995,
    pct99_9: 0.999,
};
const LEVEL_PERCENTILES: Levels<f64> = Levels {
    pct10: 0.1,
    pct20: 0.2,
    pct30: 0.3,
    pct40: 0.4,
    pct50: 0.5,
    pct60: 0.6,
    pct70: 0.7,
    pct80: 0.8,
    pct90: 0.9,
};
const WEIGHTED_URPD_VERSION: Version = Version::TWO;
const WEIGHTED_URPD_VERSION_FILE: &str = "bedrock_urpd.version";

type Thresholds = Modes<Option<Percentiles<f64>>>;
type ModeWeights = Modes<Option<AgeRange<f64>>>;
type WeightedUrpdNames = AggregateCohorts<StoredWeights<String>>;

#[derive(Default)]
struct StoredWeights<T> {
    cointime: T,
    coinflow: T,
}

impl<T> StoredWeights<T> {
    fn from_fn(mut create: impl FnMut(UrpdWeight) -> T) -> Self {
        Self {
            cointime: create(UrpdWeight::Cointime),
            coinflow: create(UrpdWeight::Coinflow),
        }
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.cointime, &self.coinflow].into_iter()
    }
}

struct AggregateCohorts<T> {
    all: T,
    term: ByTerm<T>,
}

struct WeightedMasses {
    all: WeightedModes<f64>,
    term: ByTerm<StoredWeights<f64>>,
}

impl Default for WeightedMasses {
    fn default() -> Self {
        Self {
            all: WeightedModes::from_fn(|_| 0.0),
            term: ByTerm::default(),
        }
    }
}

type WeightedUrpd = BTreeMap<CentsCompact, WeightedMasses>;

struct DayUrpds {
    raw: UrpdRaw,
    all: WeightedModes<UrpdRaw>,
    term: ByTerm<StoredWeights<UrpdRaw>>,
}

impl DayUrpds {
    fn mode(&self, mode: ModeId) -> &UrpdRaw {
        match mode {
            ModeId::Raw => &self.raw,
            _ => self.all.select(mode.weighted().expect("weighted mode")),
        }
    }
}

struct ModeResult {
    loss_threshold: Percentiles<StoredF64>,
    prices: PriceBands<Cents>,
}

struct DayResult {
    by_mode: Modes<ModeResult>,
}

impl DayResult {
    fn from_thresholds(thresholds: &Thresholds) -> Self {
        Self {
            by_mode: Modes::from_fn(|mode| ModeResult {
                loss_threshold: match thresholds.select(mode) {
                    Some(values) => Percentiles::from_fn(|percentile| {
                        StoredF64::from(*percentile.select(values))
                    }),
                    None => Percentiles::from_fn(|_| StoredF64::NAN),
                },
                prices: PriceBands::from_fn(|_| Cents::NAN),
            }),
        }
    }
}

struct Calibration {
    histories: Modes<Vec<f64>>,
}

impl Calibration {
    fn from_sources<T, U>(
        raw: &impl ReadableVec<Day1, Option<T>>,
        weighted: &WeightedModes<&dyn ReadableVec<Day1, Option<U>>>,
        end: usize,
    ) -> Self
    where
        T: VecValue,
        U: VecValue,
        f64: From<T> + From<U>,
    {
        let mut histories = Modes::from_fn(|_| Vec::new());
        histories.raw = collect_loss_history(raw, end);
        for id in WeightedModeId::ALL {
            let source = weighted.select(id);
            *histories.select_mut(id.mode()) = collect_loss_history(*source, end);
        }
        Self { histories }
    }

    fn thresholds(&self, current: &Modes<Option<f64>>) -> Thresholds {
        Modes::from_fn(|mode| {
            let history = self.histories.select(mode);
            (current.select(mode).is_some() && history.len() >= MIN_CALIBRATION_DAYS).then(|| {
                Percentiles::from_fn(|percentile| {
                    quantile(history, *percentile.select(&PERCENTILES)).expect("non-empty history")
                })
            })
        })
    }

    fn observe(&mut self, shares: Modes<Option<f64>>) {
        for mode in ModeId::ALL {
            let history = self.histories.select_mut(mode);
            let share = *shares.select(mode);
            if let Some(share) = share {
                insert_sorted(history, share.clamp(0.0, 1.0));
            }
        }
    }
}

impl ModeVecs {
    fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 2] {
        [self.loss_threshold.stored_mut(), self.prices.stored_mut()]
    }

    fn push(&mut self, result: &ModeResult) {
        self.loss_threshold.push(LossPercentileId::from_fn(|id| {
            *id.select(&result.loss_threshold)
        }));
        self.prices
            .push(PriceBandId::from_fn(|id| *id.select(&result.prices)));
    }
}

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        distribution: &distribution::Vecs,
        utxo_states: &UTXOStates,
        frameworks: &frameworks::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let cointime = &frameworks.cointime;
        let coinflow = &frameworks.coinflow;
        let cointime_wakefulness =
            AgeRange::from_fn(|id| &id.select(&cointime.age_range.activity.wakefulness).day1);
        let age_supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution.cohorts.supply.total.cohorts.age.range)
                .sats
                .day1
                .0
        });
        let coinflow_mobility = AgeRange::from_fn(|id| {
            &id.select(&coinflow.age_range.spending_exposure.mobility)
                .day1
                .0
        });
        let coinflow_spending_rate =
            AgeRange::from_fn(|id| &id.select(&coinflow.age_range.spending_rate).day1);
        let raw_loss_share = &distribution
            .cohorts
            .relative
            .supply_profitability_shares
            .supply_in_loss_share
            .all
            .ppm
            .day1
            .0;
        let weighted_loss_shares =
            WeightedModes::from_fn(|mode| -> &dyn ReadableVec<Day1, Option<StoredF64>> {
                match mode {
                    WeightedModeId::Cointime => &cointime.supply.active_supply_in_loss_share.day1,
                    WeightedModeId::Coinflow => &coinflow.all.supply_in_loss_share.day1.0,
                    WeightedModeId::Coinflow8Y => {
                        &coinflow.all.horizon._8y.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow4Y => {
                        &coinflow.all.horizon._4y.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow2Y => {
                        &coinflow.all.horizon._2y.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow1Y => {
                        &coinflow.all.horizon._1y.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow6M => {
                        &coinflow.all.horizon._6m.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow3M => {
                        &coinflow.all.horizon._3m.supply_in_loss_share.day1.0
                    }
                    WeightedModeId::Coinflow1M => {
                        &coinflow.all.horizon._1m.supply_in_loss_share.day1.0
                    }
                }
            });
        let weighted_urpd_names = weighted_urpd_names();

        let weighted_urpd_source_version: Version = std::iter::once(WEIGHTED_URPD_VERSION)
            .chain(std::iter::once(indexes.day1.date.version()))
            .chain(std::iter::once(distribution.supply_state.version()))
            .chain(age_supplies.iter().map(|vec| vec.version()))
            .chain(cointime_wakefulness.iter().map(|vec| vec.version()))
            .chain(coinflow_mobility.iter().map(|vec| vec.version()))
            .sum();
        let source_version = Version::combine_all(
            std::iter::once(weighted_urpd_source_version)
                .chain(coinflow_spending_rate.iter().map(|vec| vec.version()))
                .chain(std::iter::once(raw_loss_share.version()))
                .chain(weighted_loss_shares.iter().map(|vec| vec.version())),
        );

        for vec in self.stored_vecs_mut() {
            validate_any_computed_version_or_reset(vec, source_version)?;
        }

        let source_end = std::iter::once(indexes.day1.date.len())
            .chain(std::iter::once(raw_loss_share.len()))
            .chain(weighted_loss_shares.iter().map(|vec| vec.len()))
            .chain(age_supplies.iter().map(|vec| vec.len()))
            .chain(cointime_wakefulness.iter().map(|vec| vec.len()))
            .chain(coinflow_mobility.iter().map(|vec| vec.len()))
            .chain(coinflow_spending_rate.iter().map(|vec| vec.len()))
            .min()
            .unwrap_or_default();
        let recompute_from = recompute_day(indexer, indexes)
            .map(usize::from)
            .unwrap_or_default();
        let weighted_urpd_is_current =
            read_weighted_urpd_version(&self.states_path)? == Some(weighted_urpd_source_version);
        if !weighted_urpd_is_current {
            reset_weighted_urpds(&self.states_path, &weighted_urpd_names)?;
        }
        let weighted_urpd_start = if weighted_urpd_is_current {
            recompute_from
        } else {
            0
        };
        let start = self
            .minimum_len()
            .min(recompute_from)
            .min(weighted_urpd_start)
            .min(source_end);

        for vec in self.stored_vecs_mut() {
            vec.any_truncate_if_needed_at(start)?;
        }

        let mut calibration =
            Calibration::from_sources(raw_loss_share, &weighted_loss_shares, start);
        let bounds = age_bounds_days();

        for day_index in start..source_end {
            let day = Day1::from(day_index);
            let loss_shares = collect_loss_shares(raw_loss_share, &weighted_loss_shares, day);
            let thresholds = calibration.thresholds(&loss_shares);
            let mut result = DayResult::from_thresholds(&thresholds);

            let needs_evaluation = thresholds.iter().any(Option::is_some);
            let needs_rebuild = !weighted_urpd_is_current || day_index >= recompute_from;
            if let Some(date) = indexes.day1.date.collect_one(day)
                && (needs_rebuild || needs_evaluation)
            {
                let weights = mode_weights(
                    day,
                    &age_supplies,
                    &cointime_wakefulness,
                    &coinflow_mobility,
                    &coinflow_spending_rate,
                    &bounds,
                );
                let urpds = if day_index + 1 == source_end {
                    Some(build_current_day_urpds(utxo_states, &weights))
                } else if UrpdRaw::path(&distribution.states_path, UTXO_ALL_NAME.id, date)
                    .try_exists()?
                {
                    Some(read_day_urpds(&distribution.states_path, date, &weights)?)
                } else {
                    None
                };

                if let Some(urpds) = urpds {
                    if needs_rebuild {
                        write_weighted_day_urpds(
                            &self.states_path,
                            &weighted_urpd_names,
                            date,
                            &urpds,
                        )?;
                    }
                    if needs_evaluation {
                        evaluate_day(&urpds, &thresholds, &mut result);
                    }
                }
            }
            calibration.observe(loss_shares);

            for mode in ModeId::ALL {
                let output = self.modes.select_mut(mode);
                let mode_result = result.by_mode.select(mode);
                output.push(mode_result);
            }

            if (day_index + 1).is_multiple_of(WRITE_INTERVAL_DAYS) || day_index + 1 == source_end {
                let _lock = exit.lock();
                for vec in self.stored_vecs_mut() {
                    vec.write()?;
                }
            }
        }

        if !weighted_urpd_is_current {
            let _lock = exit.lock();
            fs::create_dir_all(&self.states_path)?;
            weighted_urpd_source_version
                .write(&self.states_path.join(WEIGHTED_URPD_VERSION_FILE))?;
        }

        Ok(())
    }

    fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.modes.iter_mut().flat_map(ModeVecs::stored_vecs_mut)
    }

    fn minimum_len(&mut self) -> usize {
        self.stored_vecs_mut()
            .map(|vec| vec.len())
            .min()
            .unwrap_or_default()
    }
}

fn collect_loss_history<T>(
    source: &(impl ReadableVec<Day1, Option<T>> + ?Sized),
    end: usize,
) -> Vec<f64>
where
    T: VecValue,
    f64: From<T>,
{
    let mut history = Vec::with_capacity(end);
    source.for_each_range_dyn_at(0, end, &mut |value| {
        if let Some(value) = value.map(f64::from).filter(|value| value.is_finite()) {
            history.push(value);
        }
    });
    history.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    history
}

fn collect_loss_share<T>(
    source: &(impl ReadableVec<Day1, Option<T>> + ?Sized),
    day: Day1,
) -> Option<f64>
where
    T: VecValue,
    f64: From<T>,
{
    source
        .collect_one(day)
        .flatten()
        .map(f64::from)
        .filter(|value| value.is_finite())
}

fn collect_loss_shares<T, U>(
    raw: &impl ReadableVec<Day1, Option<T>>,
    weighted: &WeightedModes<&dyn ReadableVec<Day1, Option<U>>>,
    day: Day1,
) -> Modes<Option<f64>>
where
    T: VecValue,
    U: VecValue,
    f64: From<T> + From<U>,
{
    let mut shares = Modes::from_fn(|_| None);
    shares.raw = collect_loss_share(raw, day);
    for id in WeightedModeId::ALL {
        let source = weighted.select(id);
        *shares.select_mut(id.mode()) = collect_loss_share(*source, day);
    }
    shares
}

fn recompute_day(indexer: &Indexer, indexes: &indexes::Vecs) -> Option<Day1> {
    let starting_height = indexer.safe_lengths().height;
    indexes
        .height
        .day1
        .collect_one(starting_height)
        .or_else(|| {
            starting_height
                .decremented()
                .and_then(|height| indexes.height.day1.collect_one(height))
        })
}

fn mode_weights(
    day: Day1,
    age_supplies: &AgeRange<&impl ReadableVec<Day1, Option<Sats>>>,
    cointime_wakefulness: &AgeRange<&impl ReadableVec<Day1, Option<StoredF64>>>,
    coinflow_mobility: &AgeRange<&impl ReadableVec<Day1, Option<StoredF64>>>,
    coinflow_spending_rate: &AgeRange<&impl ReadableVec<Day1, Option<StoredF64>>>,
    bounds: &AgeRange<AgeBand>,
) -> ModeWeights {
    debug_assert_eq!(WeightedModeId::COINFLOW_HORIZONS.len(), HORIZON_COUNT);
    let mut weights = Modes::from_fn(|_| None);
    weights.raw = Some(AgeRange::from_fn(|_| 1.0));
    weights.cointime = collect_age_values(cointime_wakefulness, age_supplies, day)
        .map(|values| AgeRange::from_fn(|id| (*id.select(&values)).clamp(0.0, 1.0)));
    weights.coinflow = collect_age_values(coinflow_mobility, age_supplies, day)
        .map(|values| AgeRange::from_fn(|id| (*id.select(&values)).clamp(0.0, 1.0)));

    if let Some(hazards) = collect_age_values(coinflow_spending_rate, age_supplies, day) {
        let hazards = AgeRange::from_fn(|id| (*id.select(&hazards)).max(0.0));
        for (id, horizon) in WeightedModeId::COINFLOW_HORIZONS
            .into_iter()
            .zip(HorizonId::ALL.into_iter().map(HorizonId::days))
        {
            *weights.select_mut(id.mode()) = Some(AgeRange::from_fn(|age| {
                horizon_mobility(&hazards, age, horizon, bounds)
            }));
        }
    }
    weights
}

fn collect_age_values<T>(
    sources: &AgeRange<&impl ReadableVec<Day1, Option<T>>>,
    supplies: &AgeRange<&impl ReadableVec<Day1, Option<Sats>>>,
    day: Day1,
) -> Option<AgeRange<f64>>
where
    T: VecValue,
    f64: From<T>,
{
    let mut values = AgeRange::default();
    for &id in AgeRangeId::ALL {
        let supply = id.select(supplies).collect_one(day).flatten()?;
        *id.select_mut(&mut values) =
            resolve_age_value(id.select(sources).collect_one(day).flatten(), supply)?;
    }
    Some(values)
}

pub(super) fn resolve_age_value<T>(value: Option<T>, supply: Sats) -> Option<f64>
where
    f64: From<T>,
{
    match value.map(f64::from) {
        Some(value) if value.is_finite() => Some(value),
        _ if supply == Sats::ZERO => Some(0.0),
        _ => None,
    }
}

fn read_weighted_urpd_version(states_path: &Path) -> Result<Option<Version>> {
    let path = states_path.join(WEIGHTED_URPD_VERSION_FILE);
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(Version::try_from(path.as_path())?))
}

fn reset_weighted_urpds(states_path: &Path, weighted_names: &WeightedUrpdNames) -> Result<()> {
    for name in weighted_names
        .all
        .iter()
        .chain(weighted_names.term.iter().flat_map(StoredWeights::iter))
    {
        remove_urpd_dir(states_path, name)?;
    }
    for id in WeightedModeId::COINFLOW_HORIZONS {
        remove_urpd_dir(states_path, &format!("bedrock_{}", id.mode().name()))?;
    }
    Ok(())
}

fn remove_urpd_dir(states_path: &Path, name: &str) -> Result<()> {
    let path = UrpdRaw::dir(states_path, name);
    match fs::remove_dir_all(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Error::new(
            error.kind(),
            format!("Cannot reset URPD '{}': {error}", path.display()),
        )
        .into()),
    }
}

fn read_day_urpds(
    distribution_states_path: &Path,
    date: Date,
    weights: &ModeWeights,
) -> Result<DayUrpds> {
    let raw = UrpdRaw::read(distribution_states_path, UTXO_ALL_NAME.id, date)?;
    let mut weighted = WeightedUrpd::new();

    for &age in AgeRangeId::ALL {
        let cohort = CohortContext::Utxo.prefixed(age.name().id);
        let source = UrpdRaw::read(distribution_states_path, &cohort, date)?;
        let is_short = TERM_FILTERS.short.includes(age.filter());

        for (price, sats) in source.map {
            add_weighted_entry(&mut weighted, price, sats, age, is_short, weights);
        }
    }

    Ok(finalize_day_urpds(raw, weighted))
}

fn build_current_day_urpds(utxos: &UTXOStates, weights: &ModeWeights) -> DayUrpds {
    build_urpds_from_age_entries(utxos.age_range_urpd_entries(), weights)
}

fn build_urpds_from_age_entries(
    entries: impl IntoIterator<Item = (AgeRangeId, CentsCompact, Sats)>,
    weights: &ModeWeights,
) -> DayUrpds {
    let mut raw = UrpdRaw::default();
    let mut weighted = WeightedUrpd::new();

    for (age, price, sats) in entries {
        *raw.map.entry(price).or_default() += sats;
        let is_short = TERM_FILTERS.short.includes(age.filter());
        add_weighted_entry(&mut weighted, price, sats, age, is_short, weights);
    }

    finalize_day_urpds(raw, weighted)
}

fn add_weighted_entry(
    weighted: &mut WeightedUrpd,
    price: CentsCompact,
    sats: Sats,
    age: AgeRangeId,
    is_short: bool,
    weights: &ModeWeights,
) {
    let mass = u64::from(sats) as f64;
    let bucket = weighted.entry(price).or_default();
    for id in WeightedModeId::ALL {
        let mode = id.mode();
        if let Some(mode_weights) = weights.select(mode) {
            let weighted_mass = mass * *age.select(mode_weights);
            *bucket.all.select_mut(id) += weighted_mass;
            let term = if is_short {
                &mut bucket.term.short
            } else {
                &mut bucket.term.long
            };
            match mode {
                ModeId::Cointime => term.cointime += weighted_mass,
                ModeId::Coinflow => term.coinflow += weighted_mass,
                _ => {}
            }
        }
    }
}

fn finalize_day_urpds(raw: UrpdRaw, weighted: WeightedUrpd) -> DayUrpds {
    let mut all = WeightedModes::from_fn(|_| UrpdRaw::default());
    let mut term = ByTerm::<StoredWeights<UrpdRaw>>::default();

    for (price, masses) in weighted {
        for id in WeightedModeId::ALL {
            let distribution = all.select_mut(id);
            let sats = floor_weighted_sats(*masses.all.select(id));
            if sats != Sats::ZERO {
                distribution.map.insert(price, sats);
            }
        }
        insert_stored_weighted_masses(price, &mut term.short, &masses.term.short);
        insert_stored_weighted_masses(price, &mut term.long, &masses.term.long);
    }

    DayUrpds { raw, all, term }
}

fn insert_stored_weighted_masses(
    price: CentsCompact,
    distributions: &mut StoredWeights<UrpdRaw>,
    masses: &StoredWeights<f64>,
) {
    insert_weighted_mass(price, &mut distributions.cointime, masses.cointime);
    insert_weighted_mass(price, &mut distributions.coinflow, masses.coinflow);
}

fn insert_weighted_mass(price: CentsCompact, distribution: &mut UrpdRaw, mass: f64) {
    let sats = floor_weighted_sats(mass);
    if sats != Sats::ZERO {
        distribution.map.insert(price, sats);
    }
}

fn floor_weighted_sats(mass: f64) -> Sats {
    debug_assert!(mass.is_finite() && mass >= 0.0);
    Sats::from(mass.floor() as u64)
}

fn write_weighted_day_urpds(
    models_states_path: &Path,
    weighted_names: &WeightedUrpdNames,
    date: Date,
    urpds: &DayUrpds,
) -> Result<()> {
    write_stored_weights(
        models_states_path,
        &weighted_names.all,
        date,
        &urpds.all.cointime,
        &urpds.all.coinflow,
    )?;
    write_stored_weights(
        models_states_path,
        &weighted_names.term.short,
        date,
        &urpds.term.short.cointime,
        &urpds.term.short.coinflow,
    )?;
    write_stored_weights(
        models_states_path,
        &weighted_names.term.long,
        date,
        &urpds.term.long.cointime,
        &urpds.term.long.coinflow,
    )
}

fn write_stored_weights(
    models_states_path: &Path,
    names: &StoredWeights<String>,
    date: Date,
    cointime: &UrpdRaw,
    coinflow: &UrpdRaw,
) -> Result<()> {
    write_weighted_urpd(models_states_path, &names.cointime, date, cointime)?;
    write_weighted_urpd(models_states_path, &names.coinflow, date, coinflow)
}

fn write_weighted_urpd(
    models_states_path: &Path,
    name: &str,
    date: Date,
    distribution: &UrpdRaw,
) -> Result<()> {
    UrpdRaw::write(
        models_states_path,
        name,
        date,
        distribution.map.iter().map(|(&price, &sats)| (price, sats)),
    )
}

fn weighted_urpd_names() -> WeightedUrpdNames {
    AggregateCohorts {
        all: StoredWeights::from_fn(|weight| weighted_urpd_name(weight, UTXO_ALL_NAME.id)),
        term: ByTerm {
            short: StoredWeights::from_fn(|weight| weighted_urpd_name(weight, TERM_NAMES.short.id)),
            long: StoredWeights::from_fn(|weight| weighted_urpd_name(weight, TERM_NAMES.long.id)),
        },
    }
}

fn evaluate_day(urpds: &DayUrpds, thresholds: &Thresholds, result: &mut DayResult) {
    for mode in ModeId::ALL {
        let urpd = urpds.mode(mode);
        let denominator = urpd.map.values().copied().map(u64::from).sum::<u64>();
        let Some(thresholds) = thresholds.select(mode) else {
            continue;
        };
        if denominator == 0
            || !urpd
                .map
                .iter()
                .any(|(price, sats)| price.inner() != 0 && *sats != Sats::ZERO)
        {
            continue;
        }

        let mut remaining_loss = denominator;
        let mut floors = Percentiles::from_fn(|_| Cents::NAN);
        let mut p95_floor = None;
        for (price, sats) in &urpd.map {
            remaining_loss -= u64::from(*sats);
            let remaining_share = remaining_loss as f64 / denominator as f64;
            for &percentile in LossPercentileId::ALL {
                let floor = percentile.select_mut(&mut floors);
                if floor.is_nan() && remaining_share <= *percentile.select(thresholds) {
                    *floor = Cents::from(*price);
                    if percentile == LossPercentileId::Pct95 {
                        p95_floor = Some(*price);
                    }
                }
            }
            if floors.iter().all(|floor| !floor.is_nan()) {
                break;
            }
        }
        let mode_result = result.by_mode.select_mut(mode);
        mode_result.prices.floor = floors;
        if let Some(p95_floor) = p95_floor {
            mode_result.prices.level = conditional_levels(urpd, p95_floor);
        }
    }
}

fn conditional_levels(urpd: &UrpdRaw, lower: CentsCompact) -> Levels<Cents> {
    let mut levels = Levels::from_fn(|_| Cents::NAN);
    let total = urpd
        .map
        .range(lower..)
        .map(|(_, sats)| u64::from(*sats))
        .sum::<u64>();
    if total == 0 {
        return levels;
    }

    let mut cumulative = 0_u64;
    let mut percentiles = LEVEL_IDS.iter().copied().peekable();
    for (price, sats) in urpd.map.range(lower..) {
        let sats = u64::from(*sats);
        if sats == 0 {
            continue;
        }
        cumulative += sats;
        while let Some(percentile) = percentiles.peek().copied()
            && cumulative as f64 >= total as f64 * *percentile.select(&LEVEL_PERCENTILES)
        {
            *percentile.select_mut(&mut levels) = Cents::from(*price);
            percentiles.next();
        }
        if percentiles.peek().is_none() {
            break;
        }
    }
    levels
}

fn quantile(sorted: &[f64], percentile: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let position = percentile.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let fraction = position - lower as f64;
    Some(sorted[lower] * (1.0 - fraction) + sorted[upper] * fraction)
}

fn insert_sorted(values: &mut Vec<f64>, value: f64) {
    let index = values
        .binary_search_by(|candidate| candidate.partial_cmp(&value).unwrap_or(Ordering::Less))
        .unwrap_or_else(|index| index);
    values.insert(index, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repeated_day_urpds<const N: usize>(entries: [(u32, u64); N]) -> DayUrpds {
        let map = entries
            .into_iter()
            .map(|(price, sats)| (CentsCompact::new(price), Sats::from(sats)))
            .collect::<BTreeMap<_, _>>();
        DayUrpds {
            raw: UrpdRaw { map: map.clone() },
            all: WeightedModes::from_fn(|_| UrpdRaw { map: map.clone() }),
            term: ByTerm {
                short: StoredWeights::from_fn(|_| UrpdRaw { map: map.clone() }),
                long: StoredWeights::from_fn(|_| UrpdRaw { map: map.clone() }),
            },
        }
    }

    #[test]
    fn quantile_linearly_interpolates() {
        assert_eq!(quantile(&[0.0, 1.0], 0.95), Some(0.95));
        assert_eq!(quantile(&[], 0.95), None);
    }

    #[test]
    fn empty_age_cohort_uses_zero_weight() {
        assert_eq!(resolve_age_value::<StoredF64>(None, Sats::ZERO), Some(0.0));
        assert_eq!(
            resolve_age_value(Some(StoredF64::NAN), Sats::ZERO),
            Some(0.0)
        );
    }

    #[test]
    fn non_empty_age_cohort_requires_finite_weight() {
        let supply = Sats::from(1_u64);
        assert_eq!(resolve_age_value::<StoredF64>(None, supply), None);
        assert_eq!(resolve_age_value(Some(StoredF64::NAN), supply), None);
        assert_eq!(
            resolve_age_value(Some(StoredF64::from(0.25)), supply),
            Some(0.25)
        );
    }

    #[test]
    fn daily_loss_share_calibrates_the_floor() {
        let urpds = repeated_day_urpds([(100, 50), (200, 50)]);
        let mut calibration = Calibration {
            histories: Modes::from_fn(|_| vec![0.5; MIN_CALIBRATION_DAYS]),
        };
        let shares = Modes::from_fn(|_| Some(0.5));
        let thresholds = calibration.thresholds(&shares);
        let mut result = DayResult::from_thresholds(&thresholds);
        evaluate_day(&urpds, &thresholds, &mut result);
        calibration.observe(shares);
        let result = &result.by_mode.coinflow;

        assert_eq!(
            result.loss_threshold,
            Percentiles::from_fn(|_| StoredF64::from(0.5))
        );
        assert_eq!(
            result.prices.floor,
            Percentiles::from_fn(|_| Cents::new(100))
        );
        assert_eq!(
            result.prices.level,
            Levels {
                pct10: Cents::new(100),
                pct20: Cents::new(100),
                pct30: Cents::new(100),
                pct40: Cents::new(100),
                pct50: Cents::new(100),
                pct60: Cents::new(200),
                pct70: Cents::new(200),
                pct80: Cents::new(200),
                pct90: Cents::new(200),
            }
        );
        assert_eq!(
            calibration.histories.coinflow.len(),
            MIN_CALIBRATION_DAYS + 1
        );
    }

    #[test]
    fn zero_cost_distribution_stays_missing() {
        let urpds = repeated_day_urpds([(0, 100)]);
        let mut calibration = Calibration {
            histories: Modes::from_fn(|_| vec![0.5; MIN_CALIBRATION_DAYS]),
        };
        let shares = Modes::from_fn(|_| Some(1.0));
        let thresholds = calibration.thresholds(&shares);
        let mut result = DayResult::from_thresholds(&thresholds);
        evaluate_day(&urpds, &thresholds, &mut result);
        calibration.observe(shares);
        let result = &result.by_mode.raw;

        assert_eq!(result.loss_threshold.pct95, StoredF64::from(0.5));
        assert!(result.prices.floor.pct95.is_nan());
        assert_eq!(calibration.histories.raw.len(), MIN_CALIBRATION_DAYS + 1);
    }

    #[test]
    fn missing_framework_share_does_not_update_history() {
        let mut calibration = Calibration {
            histories: Modes::from_fn(|_| Vec::new()),
        };
        let shares = Modes::from_fn(|_| None);
        let thresholds = calibration.thresholds(&shares);
        calibration.observe(shares);

        assert!(thresholds.iter().all(Option::is_none));
        assert!(calibration.histories.raw.is_empty());
    }

    #[test]
    fn weighted_sats_are_floored_after_summing() {
        let summed_mass = 0.6 + 0.6;
        assert_eq!(floor_weighted_sats(summed_mass), Sats::from(1_u64));
        assert_eq!(floor_weighted_sats(0.6), Sats::ZERO);

        let weighted = BTreeMap::from([(
            CentsCompact::new(100),
            WeightedMasses {
                all: WeightedModes::from_fn(|_| summed_mass),
                term: ByTerm {
                    short: StoredWeights::from_fn(|_| 0.6),
                    long: StoredWeights::from_fn(|_| 0.6),
                },
            },
        )]);
        let urpds = finalize_day_urpds(UrpdRaw::default(), weighted);

        assert_eq!(
            urpds.all.cointime.map[&CentsCompact::new(100)],
            Sats::from(1_u64)
        );
        assert!(
            !urpds
                .term
                .short
                .cointime
                .map
                .contains_key(&CentsCompact::new(100))
        );
    }

    #[test]
    fn current_day_entries_build_raw_and_weighted_urpds() {
        let weights = Modes::from_fn(|_| Some(AgeRange::from_fn(|_| 0.5)));
        let price = CentsCompact::new(100);
        let urpds = build_urpds_from_age_entries(
            [
                (AgeRangeId::Under1H, price, Sats::from(3_u64)),
                (AgeRangeId::From5MTo6M, price, Sats::from(5_u64)),
            ],
            &weights,
        );

        assert_eq!(urpds.raw.map[&price], Sats::from(8_u64));
        assert_eq!(urpds.all.cointime.map[&price], Sats::from(4_u64));
        assert_eq!(urpds.term.short.cointime.map[&price], Sats::from(1_u64));
        assert_eq!(urpds.term.long.cointime.map[&price], Sats::from(2_u64));
    }

    #[test]
    fn stores_only_cointime_and_coinflow_for_all_sth_and_lth() {
        let root =
            std::env::temp_dir().join(format!("brk-bedrock-urpd-file-{}", std::process::id()));
        let distribution_states = root.join("distribution");
        let models_states = root.join("models");
        let date = Date::new(2026, 8, 4);
        let names = weighted_urpd_names();
        let expected = repeated_day_urpds([(100, 21), (200, 34)]);
        assert_eq!(names.all.cointime, "bedrock_cointime");
        assert_eq!(names.all.coinflow, "bedrock_coinflow");
        assert_eq!(names.term.short.cointime, "bedrock_cointime_sth");
        assert_eq!(names.term.long.coinflow, "bedrock_coinflow_lth");

        UrpdRaw::write(
            &distribution_states,
            UTXO_ALL_NAME.id,
            date,
            expected.raw.map.iter().map(|(&price, &sats)| (price, sats)),
        )
        .unwrap();
        write_weighted_day_urpds(&models_states, &names, date, &expected).unwrap();

        let assert_stored = |name: &str, expected: &UrpdRaw| {
            assert_eq!(
                UrpdRaw::read(&models_states, name, date).unwrap().map,
                expected.map
            );
        };
        assert_stored(&names.all.cointime, &expected.all.cointime);
        assert_stored(&names.all.coinflow, &expected.all.coinflow);
        assert_stored(&names.term.short.cointime, &expected.term.short.cointime);
        assert_stored(&names.term.short.coinflow, &expected.term.short.coinflow);
        assert_stored(&names.term.long.cointime, &expected.term.long.cointime);
        assert_stored(&names.term.long.coinflow, &expected.term.long.coinflow);
        assert!(!UrpdRaw::path(&models_states, "bedrock_raw", date).exists());
        assert!(!UrpdRaw::path(&models_states, "bedrock_coinflow_8y", date).exists());

        UrpdRaw::write(
            &models_states,
            "bedrock_coinflow_8y",
            date,
            expected.raw.map.iter().map(|(&price, &sats)| (price, sats)),
        )
        .unwrap();
        reset_weighted_urpds(&models_states, &names).unwrap();
        assert!(UrpdRaw::path(&distribution_states, UTXO_ALL_NAME.id, date).exists());
        assert!(!UrpdRaw::path(&models_states, "bedrock_coinflow_8y", date).exists());
        assert!(
            names
                .all
                .iter()
                .chain(names.term.iter().flat_map(StoredWeights::iter))
                .all(|name| !UrpdRaw::path(&models_states, name, date).exists())
        );

        std::fs::remove_dir_all(root).unwrap();
    }
}

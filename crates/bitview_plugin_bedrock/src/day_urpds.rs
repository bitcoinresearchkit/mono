use brk_error::Result;

use std::{
    collections::BTreeMap,
    fs,
    io::{Error, ErrorKind},
    path::Path,
};

use bitview_cohort::{
    AgeRangeId, ByTerm, TERM_FILTERS, TERM_NAMES, UTXO_ALL_NAME, UTXOAggregate, UTXOAggregateId,
};
use bitview_plugin_distribution::{AgeRangeUrpds, UTXOStates};
use brk_types::{CentsCompact, Date, Sats, UrpdRaw, UrpdWeight, Version};
use vecdb::ColumnId;

use super::{ModeId, ModeWeights, WeightedModeId, WeightedModes, WeightedPair, WeightedUrpdNames};

const VERSION_FILE: &str = "bedrock_urpd.version";

struct WeightedMasses {
    all: WeightedModes<f64>,
    term: ByTerm<WeightedPair<f64>>,
}

impl Default for WeightedMasses {
    fn default() -> Self {
        Self {
            all: WeightedModes::from_fn(|_| 0.0),
            term: ByTerm::default(),
        }
    }
}

pub struct DayUrpds {
    raw: UrpdRaw,
    all: WeightedModes<UrpdRaw>,
    term: ByTerm<WeightedPair<UrpdRaw>>,
}

impl DayUrpds {
    #[cfg(test)]
    pub fn repeated<const N: usize>(entries: [(u32, u64); N]) -> Self {
        let map = entries
            .into_iter()
            .map(|(price, sats)| (CentsCompact::new(price), Sats::from(sats)))
            .collect::<BTreeMap<_, _>>();
        Self {
            raw: UrpdRaw { map: map.clone() },
            all: WeightedModes::from_fn(|_| UrpdRaw { map: map.clone() }),
            term: ByTerm {
                short: WeightedPair::from_fn(|_| UrpdRaw { map: map.clone() }),
                long: WeightedPair::from_fn(|_| UrpdRaw { map: map.clone() }),
            },
        }
    }

    pub fn mode(&self, mode: ModeId) -> &UrpdRaw {
        match mode {
            ModeId::Raw => &self.raw,
            _ => self.all.select(mode.weighted().expect("weighted mode")),
        }
    }

    pub fn names() -> WeightedUrpdNames {
        WeightedUrpdNames::new(UTXOAggregate {
            all: WeightedPair::from_fn(|weight| Self::weighted_name(weight, UTXO_ALL_NAME.id)),
            sth: WeightedPair::from_fn(|weight| Self::weighted_name(weight, TERM_NAMES.short.id)),
            lth: WeightedPair::from_fn(|weight| Self::weighted_name(weight, TERM_NAMES.long.id)),
        })
    }

    pub fn weighted_name(weight: UrpdWeight, cohort: &str) -> String {
        debug_assert!(weight.is_weighted());
        if cohort == UTXO_ALL_NAME.id {
            format!("bedrock_{}", weight.as_str())
        } else {
            format!("bedrock_{}_{cohort}", weight.as_str())
        }
    }

    pub fn read_if_exists(
        distribution_states_path: &Path,
        date: Date,
        weights: &ModeWeights,
    ) -> Result<Option<Self>> {
        if !AgeRangeUrpds::path(distribution_states_path, date).try_exists()? {
            return Ok(None);
        }
        Self::read(distribution_states_path, date, weights).map(Some)
    }

    fn read(distribution_states_path: &Path, date: Date, weights: &ModeWeights) -> Result<Self> {
        let sources = AgeRangeUrpds::read(distribution_states_path, date)?;
        let raw = sources.aggregate(UTXOAggregateId::All);
        let mut weighted = BTreeMap::new();

        for &age in AgeRangeId::ALL {
            let is_short = TERM_FILTERS.short.includes(age.filter());

            for &(price, sats) in sources.get(age) {
                Self::add_weighted_entry(&mut weighted, price, sats, age, is_short, weights);
            }
        }

        Ok(Self::finalize(raw, weighted))
    }

    pub fn current(utxos: &UTXOStates, weights: &ModeWeights) -> Self {
        Self::from_age_entries(
            AgeRangeId::ALL.iter().copied().flat_map(|age| {
                utxos
                    .age_range_urpd_entries(age)
                    .map(move |(price, sats)| (age, price, sats))
            }),
            weights,
        )
    }

    fn from_age_entries(
        entries: impl IntoIterator<Item = (AgeRangeId, CentsCompact, Sats)>,
        weights: &ModeWeights,
    ) -> Self {
        let mut raw = UrpdRaw::default();
        let mut weighted = BTreeMap::new();

        for (age, price, sats) in entries {
            *raw.map.entry(price).or_default() += sats;
            let is_short = TERM_FILTERS.short.includes(age.filter());
            Self::add_weighted_entry(&mut weighted, price, sats, age, is_short, weights);
        }

        Self::finalize(raw, weighted)
    }

    pub fn write(&self, states_path: &Path, names: &WeightedUrpdNames, date: Date) -> Result<()> {
        Self::write_pair(
            states_path,
            &names.all,
            date,
            &self.all.cointime,
            &self.all.coinflow,
        )?;
        Self::write_pair(
            states_path,
            &names.sth,
            date,
            &self.term.short.cointime,
            &self.term.short.coinflow,
        )?;
        Self::write_pair(
            states_path,
            &names.lth,
            date,
            &self.term.long.cointime,
            &self.term.long.coinflow,
        )
    }

    pub fn stored_version(states_path: &Path) -> Result<Option<Version>> {
        let path = states_path.join(VERSION_FILE);
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(Version::try_from(path.as_path())?))
    }

    pub fn write_version(states_path: &Path, version: Version) -> Result<()> {
        fs::create_dir_all(states_path)?;
        Ok(version.write(&states_path.join(VERSION_FILE))?)
    }

    pub fn reset(states_path: &Path, names: &WeightedUrpdNames) -> Result<()> {
        for name in names.iter().flat_map(WeightedPair::iter) {
            Self::remove_dir(states_path, name)?;
        }
        for id in WeightedModeId::COINFLOW_HORIZONS {
            Self::remove_dir(states_path, &format!("bedrock_{}", id.mode().name()))?;
        }
        Ok(())
    }

    fn add_weighted_entry(
        weighted: &mut BTreeMap<CentsCompact, WeightedMasses>,
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

    fn finalize(raw: UrpdRaw, weighted: BTreeMap<CentsCompact, WeightedMasses>) -> Self {
        let mut all = WeightedModes::from_fn(|_| UrpdRaw::default());
        let mut term = ByTerm::<WeightedPair<UrpdRaw>>::default();

        for (price, masses) in weighted {
            for id in WeightedModeId::ALL {
                let distribution = all.select_mut(id);
                Self::insert_mass(price, distribution, *masses.all.select(id));
            }
            Self::insert_pair(price, &mut term.short, &masses.term.short);
            Self::insert_pair(price, &mut term.long, &masses.term.long);
        }

        Self { raw, all, term }
    }

    fn insert_pair(
        price: CentsCompact,
        distributions: &mut WeightedPair<UrpdRaw>,
        masses: &WeightedPair<f64>,
    ) {
        Self::insert_mass(price, &mut distributions.cointime, masses.cointime);
        Self::insert_mass(price, &mut distributions.coinflow, masses.coinflow);
    }

    fn insert_mass(price: CentsCompact, distribution: &mut UrpdRaw, mass: f64) {
        let sats = Self::floor_sats(mass);
        if sats != Sats::ZERO {
            distribution.map.insert(price, sats);
        }
    }

    fn floor_sats(mass: f64) -> Sats {
        debug_assert!(mass.is_finite() && mass >= 0.0);
        Sats::from(mass.floor() as u64)
    }

    fn write_pair(
        states_path: &Path,
        names: &WeightedPair<String>,
        date: Date,
        cointime: &UrpdRaw,
        coinflow: &UrpdRaw,
    ) -> Result<()> {
        Self::write_one(states_path, &names.cointime, date, cointime)?;
        Self::write_one(states_path, &names.coinflow, date, coinflow)
    }

    fn write_one(states_path: &Path, name: &str, date: Date, distribution: &UrpdRaw) -> Result<()> {
        UrpdRaw::write(
            states_path,
            name,
            date,
            distribution.map.iter().map(|(&price, &sats)| (price, sats)),
        )
    }

    fn remove_dir(states_path: &Path, name: &str) -> Result<()> {
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
}

#[cfg(test)]
mod tests {
    use bitview_cohort::{AgeRange, AgeRangeId, UTXO_ALL_NAME};
    use bitview_plugin_distribution::{AgeRangeUrpds, UTXOStates};
    use brk_types::{CentsCompact, Date, Sats, UrpdRaw};

    use super::{DayUrpds, ModeWeights};

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
        assert_eq!(urpds.term.short.cointime.map[&price], Sats::from(1_u64));
        assert_eq!(urpds.term.long.cointime.map[&price], Sats::from(2_u64));
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
    fn historical_read_uses_packed_source_without_legacy_all_file() {
        let root = tempfile::tempdir().unwrap();
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
}

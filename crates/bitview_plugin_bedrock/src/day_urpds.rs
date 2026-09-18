use std::{
    collections::BTreeMap,
    fs,
    io::{Error, ErrorKind},
    path::Path,
};

use bitview_cohort::{AgeRangeId, TERM_NAMES, Term, UTXO_ALL_NAME, UTXOAggregate, UTXOAggregateId};
use bitview_plugin_distribution::{AgeRangeUrpds, UTXOStates};
use brk_error::Result;
use brk_types::{Cents, CentsCompact, Date, PartsPerMillion32, Sats, UrpdRaw, UrpdWeight, Version};

use super::{ModeId, ModeWeights, WeightedModeId, WeightedModes, WeightedPair, WeightedUrpdNames};
use crate::{
    AgeCutoffs, CostBasisData, DensityBands, PriceBounds, SupplyDensity, capitalized_price,
};

const VERSION_FILE: &str = "bedrock_urpd.version";

struct WeightedMasses {
    all: WeightedModes<f64>,
    age: AgeCutoffs<WeightedPair<f64>>,
    long: WeightedPair<f64>,
}

impl Default for WeightedMasses {
    fn default() -> Self {
        Self {
            all: WeightedModes::from_fn(|_| 0.0),
            age: AgeCutoffs::default(),
            long: WeightedPair::default(),
        }
    }
}

pub struct DayUrpds {
    pub age_price_bounds: AgeCutoffs<PriceBounds<Cents>>,
    raw: UrpdRaw,
    all: WeightedModes<UrpdRaw>,
    age: AgeCutoffs<WeightedPair<UrpdRaw>>,
    long: WeightedPair<UrpdRaw>,
}

impl DayUrpds {
    pub fn capitalized_prices(&self) -> UTXOAggregate<WeightedPair<Cents>> {
        let price = |urpd: &UrpdRaw| {
            capitalized_price::capitalized_price(urpd.map.iter().map(|(&p, &s)| (p, s)))
        };
        UTXOAggregate {
            all: WeightedPair {
                cointime: price(&self.all.cointime),
                coinflow: price(&self.all.coinflow),
            },
            sth: WeightedPair {
                cointime: price(&self.age.under_5m.cointime),
                coinflow: price(&self.age.under_5m.coinflow),
            },
            lth: WeightedPair {
                cointime: price(&self.long.cointime),
                coinflow: price(&self.long.coinflow),
            },
        }
    }

    /// Backfill only the six new prices from version-validated saved URPDs.
    /// Missing dates stay undefined; a half-written pair is an error.
    pub fn read_capitalized_prices(
        states_path: &Path,
        names: &WeightedUrpdNames,
        date: Date,
    ) -> Result<UTXOAggregate<WeightedPair<Cents>>> {
        let read_cohort = |cohort: UTXOAggregateId| -> Result<_> {
            let names = cohort.select(names);
            let paths = [
                UrpdRaw::path(states_path, &names.cointime, date),
                UrpdRaw::path(states_path, &names.coinflow, date),
            ];
            match (paths[0].try_exists()?, paths[1].try_exists()?) {
                (false, false) => Ok(WeightedPair::from_fn(|_| Cents::NAN)),
                (true, true) => {
                    let read = |name: &str| -> Result<Cents> {
                        let bytes = UrpdRaw::read_bytes(states_path, name, date)?;
                        Ok(capitalized_price::capitalized_price(
                            UrpdRaw::deserialize_entries(&bytes)?,
                        ))
                    };
                    Ok(WeightedPair {
                        cointime: read(&names.cointime)?,
                        coinflow: read(&names.coinflow)?,
                    })
                }
                _ => Err(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "Incomplete weighted URPD pair: '{}' and '{}'",
                        paths[0].display(),
                        paths[1].display()
                    ),
                )
                .into()),
            }
        };
        Ok(UTXOAggregate {
            all: read_cohort(UTXOAggregateId::All)?,
            sth: read_cohort(UTXOAggregateId::Sth)?,
            lth: read_cohort(UTXOAggregateId::Lth)?,
        })
    }

    #[cfg(test)]
    pub fn repeated<const N: usize>(entries: [(u32, u64); N]) -> Self {
        let map = entries
            .into_iter()
            .map(|(price, sats)| (CentsCompact::new(price), Sats::from(sats)))
            .collect::<BTreeMap<_, _>>();
        let mut age_price_bounds = AgeCutoffs::default();
        for (&price, &sats) in &map {
            age_price_bounds.include(AgeRangeId::Under1H, price, sats);
        }
        Self {
            age_price_bounds,
            raw: UrpdRaw { map: map.clone() },
            all: WeightedModes::from_fn(|_| UrpdRaw { map: map.clone() }),
            age: AgeCutoffs::from_fn(|| WeightedPair::from_fn(|_| UrpdRaw { map: map.clone() })),
            long: WeightedPair::from_fn(|_| UrpdRaw { map: map.clone() }),
        }
    }

    pub fn mode(&self, mode: ModeId) -> &UrpdRaw {
        match mode {
            ModeId::Raw => &self.raw,
            _ => self.all.select(mode.weighted().expect("weighted mode")),
        }
    }

    pub fn cost_basis(&self, spot: Cents) -> UTXOAggregate<WeightedPair<CostBasisData>> {
        let compute = |urpd: &UrpdRaw| {
            CostBasisData::from_entries(urpd.map.iter().map(|(&p, &s)| (p, s)), spot)
        };
        UTXOAggregate {
            all: WeightedPair {
                cointime: compute(&self.all.cointime),
                coinflow: compute(&self.all.coinflow),
            },
            sth: WeightedPair {
                cointime: compute(&self.age.under_5m.cointime),
                coinflow: compute(&self.age.under_5m.coinflow),
            },
            lth: WeightedPair {
                cointime: compute(&self.long.cointime),
                coinflow: compute(&self.long.coinflow),
            },
        }
    }

    pub fn age_densities(
        &self,
        spot: Cents,
    ) -> AgeCutoffs<WeightedPair<DensityBands<SupplyDensity<PartsPerMillion32>>>> {
        let compute = |pair: &WeightedPair<UrpdRaw>| {
            let bands = |urpd: &UrpdRaw| {
                DensityBands::from_entries(urpd.map.iter().map(|(&p, &s)| (p, s)), spot)
            };
            WeightedPair {
                cointime: bands(&pair.cointime),
                coinflow: bands(&pair.coinflow),
            }
        };
        AgeCutoffs {
            under_4m: compute(&self.age.under_4m),
            under_5m: compute(&self.age.under_5m),
            under_6m: compute(&self.age.under_6m),
        }
    }

    pub fn read_cost_basis_if_exists(
        states_path: &Path,
        names: &WeightedUrpdNames,
        date: Date,
        spot: Cents,
    ) -> Result<Option<UTXOAggregate<WeightedPair<CostBasisData>>>> {
        let read = |names: &WeightedPair<String>| -> Result<Option<WeightedPair<CostBasisData>>> {
            let cointime_path = UrpdRaw::path(states_path, &names.cointime, date);
            let coinflow_path = UrpdRaw::path(states_path, &names.coinflow, date);
            match (cointime_path.try_exists()?, coinflow_path.try_exists()?) {
                (false, false) => return Ok(None),
                (true, true) => {}
                _ => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!(
                            "Incomplete weighted URPD pair: '{}' and '{}'",
                            cointime_path.display(),
                            coinflow_path.display()
                        ),
                    )
                    .into());
                }
            }
            let read = |name: &str| -> Result<CostBasisData> {
                let bytes = UrpdRaw::read_bytes(states_path, name, date)?;
                let entries = UrpdRaw::deserialize_entries(&bytes)?;
                Ok(CostBasisData::from_entries(entries.iter().copied(), spot))
            };
            Ok(Some(WeightedPair {
                cointime: read(&names.cointime)?,
                coinflow: read(&names.coinflow)?,
            }))
        };
        let all = read(&names.all)?;
        let sth = read(&names.sth)?;
        let lth = read(&names.lth)?;
        if all.is_none() && sth.is_none() && lth.is_none() {
            return Ok(None);
        }
        Ok(Some(UTXOAggregate {
            all: all.unwrap_or_default(),
            sth: sth.unwrap_or_default(),
            lth: lth.unwrap_or_default(),
        }))
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
        let raw = sources.aggregate(UTXOAggregateId::All)?;
        let mut weighted = BTreeMap::new();
        let mut age_price_bounds = AgeCutoffs::default();

        for &age in AgeRangeId::ALL {
            let is_short = age.term() == Term::Sth;

            for &(price, sats) in sources.get(age) {
                age_price_bounds.include(age, price, sats);
                Self::add_weighted_entry(&mut weighted, price, sats, age, is_short, weights);
            }
        }

        Ok(Self::finalize(raw, weighted, age_price_bounds))
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
        let mut age_price_bounds = AgeCutoffs::default();

        for (age, price, sats) in entries {
            age_price_bounds.include(age, price, sats);
            *raw.map.entry(price).or_default() += sats;
            let is_short = age.term() == Term::Sth;
            Self::add_weighted_entry(&mut weighted, price, sats, age, is_short, weights);
        }

        Self::finalize(raw, weighted, age_price_bounds)
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
            &self.age.under_5m.cointime,
            &self.age.under_5m.coinflow,
        )?;
        Self::write_pair(
            states_path,
            &names.lth,
            date,
            &self.long.cointime,
            &self.long.coinflow,
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
                if !matches!(mode, ModeId::Cointime | ModeId::Coinflow) {
                    continue;
                }
                for cohort in bucket
                    .age
                    .containing_mut(age)
                    .chain((!is_short).then_some(&mut bucket.long))
                {
                    match mode {
                        ModeId::Cointime => cohort.cointime += weighted_mass,
                        ModeId::Coinflow => cohort.coinflow += weighted_mass,
                        _ => {}
                    }
                }
            }
        }
    }

    fn finalize(
        raw: UrpdRaw,
        weighted: BTreeMap<CentsCompact, WeightedMasses>,
        age_price_bounds: AgeCutoffs<PriceBounds<Cents>>,
    ) -> Self {
        let all = WeightedModes::from_fn(|id| {
            Self::collect_mass(&weighted, |masses| *masses.all.select(id))
        });
        let pair = |select: fn(&WeightedMasses) -> &WeightedPair<f64>| WeightedPair {
            cointime: Self::collect_mass(&weighted, |masses| select(masses).cointime),
            coinflow: Self::collect_mass(&weighted, |masses| select(masses).coinflow),
        };
        let age = AgeCutoffs {
            under_4m: pair(|masses| &masses.age.under_4m),
            under_5m: pair(|masses| &masses.age.under_5m),
            under_6m: pair(|masses| &masses.age.under_6m),
        };
        let long = pair(|masses| &masses.long);

        Self {
            raw,
            all,
            age,
            long,
            age_price_bounds,
        }
    }

    fn collect_mass(
        weighted: &BTreeMap<CentsCompact, WeightedMasses>,
        mass: impl Fn(&WeightedMasses) -> f64,
    ) -> UrpdRaw {
        UrpdRaw {
            map: weighted
                .iter()
                .filter_map(|(&price, masses)| {
                    let sats = Self::floor_sats(mass(masses));
                    (sats != Sats::ZERO).then_some((price, sats))
                })
                .collect(),
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
#[path = "../tests/unit/day_urpds.rs"]
mod tests;

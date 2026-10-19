use bitview_cohort::{AgeAggregateId, AgeRange, AgeRangeId, CohortContext};
use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{Date, Day1};
use bitview_types::{Cohort, Urpd, UrpdAggregation, UrpdWeight};
use bitview_urpd::OriginUrpd;
use brk_types::{BlockHash, Height, Sats, Version};
use vecdb::{AnyVec, ReadableVec};

use crate::{Error, OptionData, Query, ReadPin, Result, check_bound};

mod entries;

use entries::UrpdHeader;

/// A URPD request resolved under one pin to the block it answers for: its identity, without
/// reconstructing the distribution, which [`Query::urpd_resolved`] does under the same pin.
pub struct ResolvedUrpd {
    pin: SafeLengths,
    cohort: Cohort,
    ages: &'static [AgeRangeId],
    height: Height,
    hash: BlockHash,
    aggregation: UrpdAggregation,
    weight: UrpdWeight,
    /// The summed versions of the sources the distribution reads.
    version: Version,
    settled: bool,
}

impl ResolvedUrpd {
    /// Identifies the response: the request, the block (its hash commits to the chain the
    /// distribution reconstructs) and the versions of the sources it reads.
    pub fn representation(&self) -> String {
        format!(
            "{}-{}-{}-{}-{}",
            self.version, self.cohort, self.aggregation, self.weight, self.hash
        )
    }

    /// Whether the URL pins the point below the reorg window: appends never change it.
    pub fn is_settled(&self) -> bool {
        self.settled
    }
}

impl Query {
    pub fn urpd_cohorts(&self) -> Result<Vec<Cohort>> {
        let pin = self.pin_safe_lengths()?;
        let view = self.plugins().utxo_set.view()?;
        let published = usize::from(pin.lengths().height);
        if published == 0 || view.reader(published)?.is_empty() {
            return Ok(Vec::new());
        }
        let mut cohorts: Vec<_> = AgeRangeId::ALL
            .iter()
            .filter_map(|id| Cohort::new(CohortContext::Utxo.prefixed(id.name().id)))
            .chain(
                AgeAggregateId::ALL
                    .iter()
                    .filter_map(|id| Cohort::new(id.name())),
            )
            .collect();
        cohorts.sort_unstable();
        Ok(cohorts)
    }

    pub fn resolve_urpd_at(
        &self,
        cohort: &Cohort,
        date: Date,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let pin = self.pin_safe_lengths()?;
        let end = usize::from(pin.lengths().height);
        let day = Day1::try_from(date)?;
        let (heights, settled) = self.day_heights(day, end, &pin)?;
        let height = Height::from(heights.end - 1);
        self.resolve_urpd_inner(cohort, height, aggregation, weight, pin, settled)
    }

    pub fn resolve_urpd_height(
        &self,
        cohort: &Cohort,
        height: Height,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let pin = self.pin_safe_lengths()?;
        let settled = pin.lengths().is_settled(height);
        self.resolve_urpd_inner(cohort, height, aggregation, weight, pin, settled)
    }

    pub fn resolve_urpd_latest(
        &self,
        cohort: &Cohort,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let pin = self.pin_safe_lengths()?;
        let height = pin
            .lengths()
            .last_height()
            .ok_or_else(|| Error::NotFound("No published UTXO set".into()))?;
        self.resolve_urpd_inner(cohort, height, aggregation, weight, pin, false)
    }

    fn urpd_ages(cohort: &Cohort) -> Result<&'static [AgeRangeId]> {
        if let Some(age) = AgeRangeId::from_cohort_name(CohortContext::Utxo, cohort) {
            return Ok(&AgeRangeId::ALL[age.index()..age.index() + 1]);
        }
        AgeAggregateId::from_name(cohort)
            .map(AgeAggregateId::age_range_ids)
            .ok_or_else(|| Error::NotFound(format!("Unknown URPD cohort '{cohort}'")))
    }

    fn urpd_weights(
        &self,
        ages: &[AgeRangeId],
        height: Height,
        weight: UrpdWeight,
    ) -> Result<AgeRange<f64>> {
        AgeRange::try_from_fn(|age| {
            if weight == UrpdWeight::Raw || !ages.contains(&age) {
                return Ok(1.0);
            }
            let plugins = self.plugins();
            let supply = age
                .select(&plugins.age.ranges)
                .supply
                .total
                .value
                .sats
                .height
                .collect_one(height)
                .data()?;
            match weight {
                UrpdWeight::Raw => Some(1.0),
                UrpdWeight::Cointime => plugins.cointime.urpd_weight(age, height, supply),
                UrpdWeight::Coinflow => plugins.coinflow.urpd_weight(age, height, supply),
            }
            .ok_or_else(|| Error::NotFound(format!("No {weight} weight at block {height}")))
        })
    }

    fn resolve_urpd_inner(
        &self,
        cohort: &Cohort,
        height: Height,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
        pin: ReadPin,
        settled: bool,
    ) -> Result<ResolvedUrpd> {
        let ages = Self::urpd_ages(cohort)?;
        check_bound(&pin, height, pin.lengths().height)?;
        if height >= pin.lengths().height {
            return Err(Error::NotFound("Block is not published".into()));
        }
        let hash = self
            .indexer()
            .vecs()
            .blocks
            .blockhash
            .collect_one(height)
            .data()?;
        Ok(ResolvedUrpd {
            version: self.urpd_version(ages, weight),
            pin: pin.into_inner(),
            cohort: cohort.clone(),
            ages,
            height,
            hash,
            aggregation,
            weight,
            settled,
        })
    }

    /// The summed versions of the sources a distribution reads: prices, timestamps and the UTXO
    /// set history, and for a weighted one the cohort's supplies and weights.
    fn urpd_version(&self, ages: &[AgeRangeId], weight: UrpdWeight) -> Version {
        let plugins = self.plugins();
        let base = plugins.price.spot.cents.height.version()
            + plugins.mappings.timestamp.monotonic.version()
            + plugins.utxo_set.supply.version();
        let weights = |age: &AgeRangeId| match weight {
            UrpdWeight::Raw => Version::ZERO,
            UrpdWeight::Cointime => age
                .select(&plugins.cointime.age_ranges.urpd_weight_sources())
                .version(),
            UrpdWeight::Coinflow => age
                .select(&plugins.coinflow.age_ranges.urpd_weight_sources())
                .version(),
        };
        if weight == UrpdWeight::Raw {
            return base;
        }
        base + ages
            .iter()
            .map(|age| {
                let supply = &age
                    .select(&plugins.age.ranges)
                    .supply
                    .total
                    .value
                    .sats
                    .height;
                supply.version() + weights(age)
            })
            .sum::<Version>()
    }

    /// Reconstructs the distribution `resolved` selected, under its pin.
    pub fn urpd_resolved(&self, resolved: ResolvedUrpd) -> Result<Urpd> {
        let ResolvedUrpd {
            pin,
            cohort,
            ages,
            height,
            hash,
            aggregation,
            weight,
            ..
        } = resolved;
        let pin = self.read_pin(pin);
        let end = usize::from(height) + 1;
        let plugins = self.plugins();
        let weights = self.urpd_weights(ages, height, weight)?;
        let view = plugins.utxo_set.view()?;
        let reader = view.reader(usize::from(pin.lengths().height))?;
        if end > reader.len() {
            return Err(Error::NotFound(
                "Block is outside the published UTXO set".into(),
            ));
        }
        let state = reader.state_at(end)?;
        if state.hash() != *hash {
            return Err(Error::Internal(
                "UTXO set history does not match the published chain",
            ));
        }
        let prices = plugins.price.spot.cents.height.collect_range_at(0, end);
        let timestamps = plugins
            .mappings
            .timestamp
            .monotonic
            .collect_range_at(0, end);
        let source = OriginUrpd::new(&state, &prices, &timestamps)?;
        let header = UrpdHeader {
            cohort,
            height,
            date: Date::from(timestamps[end - 1]),
            weight,
            aggregation,
            close: prices[end - 1],
        };
        Ok(if weight == UrpdWeight::Raw {
            header.build(source.project(&[], [ages]).map(|b| (b.price, b.raw[0])))
        } else {
            header.build(source.project(&[Some(&weights)], [ages]).filter_map(|b| {
                let sats = b.weighted[0][0];
                (sats != Sats::ZERO).then_some((b.price, sats))
            }))
        })
    }
}

use bitview_cohort::{AgeAggregateId, AgeRange, AgeRangeId, CohortContext};
use bitview_primitives::{CentsCompact, Date, Day1};
use bitview_types::{Cohort, UrpdAggregation, UrpdWeight};
use bitview_urpd::OriginUrpd;
use brk_types::{Cents, Height, Sats};
use vecdb::{AnyVec, ReadableVec};

use crate::{Error, OptionData, Query, Result};

mod resolved;

/// Owned inputs captured from one published block. No later file or plugin reads.
pub struct ResolvedUrpd {
    pub cohort: Cohort,
    pub height: Height,
    pub date: Date,
    pub weight: UrpdWeight,
    pub aggregation: UrpdAggregation,
    pub close: Cents,
    entries: Box<[(CentsCompact, Sats)]>,
}

impl Query {
    pub fn urpd_cohorts(&self) -> Result<Vec<Cohort>> {
        let _guard = self.read_publication()?;
        let view = self.plugins().utxo_history.view()?;
        if self.safe_lengths().height.is_zero() || view.reader()?.is_empty() {
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

    /// Calendar aliases are derived from block coverage, never snapshot filenames.
    pub fn urpd_dates_with_weight(&self, cohort: &Cohort, weight: UrpdWeight) -> Result<Vec<Date>> {
        let _guard = self.read_publication()?;
        let ages = Self::urpd_ages(cohort)?;
        let view = self.plugins().utxo_history.view()?;
        let reader = view.reader()?;
        let end = reader.len().min(usize::from(self.safe_lengths().height));
        let mut dates = Vec::new();
        for index in 0..self.plugins().mappings.day1.date.len() {
            let date = Date::from(Day1::from(index));
            if let Ok(height) = self.urpd_date_height(date, end)
                && usize::from(height) + 1 >= reader.start()
                && self.urpd_weights(ages, height, weight).is_ok()
            {
                dates.push(date);
            }
        }
        Ok(dates)
    }

    pub fn resolve_urpd_at(
        &self,
        cohort: &Cohort,
        date: Date,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let _guard = self.read_publication()?;
        let height = self.urpd_date_height(date, usize::from(self.safe_lengths().height))?;
        self.resolve_urpd_inner(cohort, height, aggregation, weight)
    }

    pub fn resolve_urpd_height(
        &self,
        cohort: &Cohort,
        height: Height,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let _guard = self.read_publication()?;
        self.resolve_urpd_inner(cohort, height, aggregation, weight)
    }

    pub fn resolve_urpd_latest(
        &self,
        cohort: &Cohort,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let _guard = self.read_publication()?;
        let height = self
            .safe_lengths()
            .last_height()
            .ok_or_else(|| Error::NotFound("No published UTXO history".into()))?;
        self.resolve_urpd_inner(cohort, height, aggregation, weight)
    }

    fn urpd_date_height(&self, date: Date, end: usize) -> Result<Height> {
        let day = Day1::try_from(date)?;
        let starts = &self.plugins().mappings.day1.first_height;
        let start = starts.collect_one(day).map(usize::from).unwrap_or(end);
        let next = starts
            .collect_one_at(usize::from(day) + 1)
            .map(usize::from)
            .unwrap_or(end)
            .min(end);
        if start >= next {
            return Err(Error::NotFound(format!("No published block on {date}")));
        }
        Ok(Height::from(next - 1))
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
                .select(&plugins.distribution_age.cohorts.supply.total.cohorts.age)
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
    ) -> Result<ResolvedUrpd> {
        let ages = Self::urpd_ages(cohort)?;
        let end = usize::from(height) + 1;
        if end > usize::from(self.safe_lengths().height) {
            return Err(Error::NotFound("Block is not published".into()));
        }
        let plugins = self.plugins();
        let weights = self.urpd_weights(ages, height, weight)?;
        let view = plugins.utxo_history.view()?;
        let reader = view.reader()?;
        if end < reader.start() || end > reader.len() {
            return Err(Error::NotFound(
                "Block is outside published UTXO history".into(),
            ));
        }
        let state = reader.state_at(end)?;
        let hash = self
            .indexer()
            .vecs()
            .blocks
            .blockhash
            .collect_one(height)
            .data()?;
        if state.hash() != *hash {
            return Err(Error::StateUpdating);
        }
        let prices = plugins.price.spot.cents.height.collect_range_at(0, end);
        let timestamps = plugins
            .mappings
            .timestamp
            .monotonic
            .collect_range_at(0, end);
        let source = OriginUrpd::new(&state, &prices, &timestamps)?;
        let entries = if weight == UrpdWeight::Raw {
            source
                .project(&[], [ages])
                .map(|b| (b.price, b.raw[0]))
                .collect()
        } else {
            source
                .project(&[Some(&weights)], [ages])
                .filter_map(|b| {
                    let sats = b.weighted[0][0];
                    (sats != Sats::ZERO).then_some((b.price, sats))
                })
                .collect()
        };
        Ok(ResolvedUrpd {
            cohort: cohort.clone(),
            height,
            date: Date::from(timestamps[end - 1]),
            weight,
            aggregation,
            close: prices[end - 1],
            entries,
        })
    }
}

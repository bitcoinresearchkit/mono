use bitview_cohort::{AgeRange, AgeRangeId, CohortContext, UTXOAggregateId};
use bitview_urpd::{AgeRangeUrpds, accumulate_masses, collect_mass, weighted_entries};
use brk_error::{Error, OptionData, Result};
use brk_types::{
    Cents, CentsCompact, Cohort, Date, Day1, Height, Sats, Term, UrpdAggregation, UrpdWeight,
};
use vecdb::{AnyVec, ReadableVec};

use crate::Query;

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
                UTXOAggregateId::ALL
                    .iter()
                    .filter_map(|id| Cohort::new(id.cohort_name().id)),
            )
            .collect();
        cohorts.sort_unstable();
        Ok(cohorts)
    }

    /// Calendar aliases are derived from block coverage, never snapshot filenames.
    pub fn urpd_dates_with_weight(&self, cohort: &Cohort, weight: UrpdWeight) -> Result<Vec<Date>> {
        let _guard = self.read_publication()?;
        Self::validate_urpd_cohort(cohort)?;
        let view = self.plugins().utxo_history.view()?;
        let reader = view.reader()?;
        let end = reader.len().min(usize::from(self.safe_lengths().height));
        let mut dates = Vec::new();
        for index in 0..self.plugins().mappings.day1.date.len() {
            let date = Date::from(Day1::from(index));
            if let Ok(height) = self.urpd_date_height(date, end)
                && usize::from(height) + 1 >= reader.start()
                && self.urpd_weights(cohort, height, weight).is_ok()
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

    fn urpd_age_range_id(cohort: &Cohort) -> Option<AgeRangeId> {
        AgeRangeId::from_cohort_name(CohortContext::Utxo, cohort)
    }

    fn urpd_aggregate_id(cohort: &Cohort) -> Option<UTXOAggregateId> {
        UTXOAggregateId::from_cohort_name(cohort)
    }

    fn validate_urpd_cohort(cohort: &Cohort) -> Result<()> {
        if Self::urpd_age_range_id(cohort).is_none() && Self::urpd_aggregate_id(cohort).is_none() {
            return Err(Error::NotFound(format!("Unknown URPD cohort '{cohort}'")));
        }
        Ok(())
    }

    fn urpd_contains(cohort: &Cohort, age: AgeRangeId) -> bool {
        if let Some(single) = Self::urpd_age_range_id(cohort) {
            return age == single;
        }
        match Self::urpd_aggregate_id(cohort).expect("validated URPD cohort") {
            UTXOAggregateId::All => true,
            UTXOAggregateId::Sth => age.term() == Term::Sth,
            UTXOAggregateId::Lth => age.term() == Term::Lth,
        }
    }

    fn urpd_weights(
        &self,
        cohort: &Cohort,
        height: Height,
        weight: UrpdWeight,
    ) -> Result<AgeRange<f64>> {
        AgeRange::try_from_fn(|age| {
            if weight == UrpdWeight::Raw || !Self::urpd_contains(cohort, age) {
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
        Self::validate_urpd_cohort(cohort)?;
        let end = usize::from(height) + 1;
        if end > usize::from(self.safe_lengths().height) {
            return Err(Error::NotFound("Block is not published".into()));
        }
        let plugins = self.plugins();
        let weights = self.urpd_weights(cohort, height, weight)?;
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
        let source = AgeRangeUrpds::from_origins(&state, &prices, &timestamps)?;
        let entries = if let Some(age) = Self::urpd_age_range_id(cohort) {
            weighted_entries(source.get(age).iter().copied(), *age.select(&weights)).collect()
        } else {
            let buckets = accumulate_masses(
                source
                    .iter()
                    .filter(|(age, _, _)| Self::urpd_contains(cohort, *age)),
                |mass: &mut f64, age, sats| *mass += u64::from(sats) as f64 * *age.select(&weights),
            );
            collect_mass(&buckets, |mass| *mass)
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

use std::{
    cmp::Ordering,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use bitview_cohort::{AgeRangeId, CohortContext, UTXO_ALL_NAME, UTXOAggregateId};
use bitview_urpd::{AgeRangeUrpds, UrpdRaw};
use brk_error::{Error, Result};
use brk_types::{Cents, Cohort, Date, Day1, UrpdAggregation, UrpdWeight};
use vecdb::{ReadableOptionVec, ReadableVec};

use crate::Query;

pub mod resolved;
use resolved::UrpdInput;

/// Owned domain inputs; no files or mutable plugin state are read after capture.
pub struct ResolvedUrpd {
    pub cohort: Cohort,
    pub date: Date,
    pub weight: UrpdWeight,
    pub aggregation: UrpdAggregation,
    pub scalar: f64,
    pub close: Cents,
    input: UrpdInput,
}

impl Query {
    /// Available cohorts for URPD.
    pub fn urpd_cohorts(&self) -> Result<Vec<Cohort>> {
        let _guard = self.read_publication()?;
        self.urpd_cohorts_inner()
    }

    fn urpd_cohorts_inner(&self) -> Result<Vec<Cohort>> {
        let states_path = &self.plugins().distribution.states_path;
        let age_range_dir = AgeRangeUrpds::dir(states_path);

        let entries = match fs::read_dir(states_path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        let mut cohorts = Vec::new();
        let mut has_age_ranges = false;
        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if entry.path() == age_range_dir {
                has_age_ranges = latest_date_in_dir(&age_range_dir)?.is_some();
                continue;
            }
            let Some(cohort) = entry.file_name().to_str().and_then(Cohort::new) else {
                continue;
            };
            if Self::urpd_age_range_id(&cohort).is_none()
                && Self::urpd_aggregate_id(&cohort).is_none()
            {
                let dir = UrpdRaw::dir(states_path, &cohort);
                if dir.try_exists()? && latest_date_in_dir(&dir)?.is_some() {
                    cohorts.push(cohort);
                }
            }
        }

        if has_age_ranges {
            cohorts.extend(
                AgeRangeId::ALL
                    .iter()
                    .filter_map(|id| Cohort::new(CohortContext::Utxo.prefixed(id.name().id))),
            );
            cohorts.extend(
                UTXOAggregateId::ALL
                    .iter()
                    .filter_map(|id| Cohort::new(id.cohort_name().id)),
            );
        }

        cohorts.sort_unstable();
        cohorts.dedup();

        Ok(cohorts)
    }

    fn urpd_dir(&self, cohort: &Cohort) -> Result<PathBuf> {
        let states_path = &self.plugins().distribution.states_path;
        let is_age_range = Self::urpd_age_range_id(cohort).is_some();
        let is_aggregate = Self::urpd_aggregate_id(cohort).is_some();
        let dir = if is_age_range || is_aggregate {
            AgeRangeUrpds::dir(states_path)
        } else {
            UrpdRaw::dir(states_path, cohort)
        };

        if !dir.try_exists()? {
            return Err(Error::NotFound("Unknown URPD cohort".into()));
        }

        Ok(dir)
    }

    /// Available dates for a cohort and weighting.
    pub fn urpd_dates_with_weight(&self, cohort: &Cohort, weight: UrpdWeight) -> Result<Vec<Date>> {
        let _guard = self.read_publication()?;
        self.urpd_dates_with_weight_inner(cohort, weight)
    }

    fn urpd_dates_with_weight_inner(
        &self,
        cohort: &Cohort,
        weight: UrpdWeight,
    ) -> Result<Vec<Date>> {
        if weight == UrpdWeight::Raw {
            return dates_in_dir(&self.urpd_dir(cohort)?);
        }

        let mut dates = Vec::new();
        self.visit_weighted_urpd_dates(cohort, weight, |date| dates.push(date))?;
        Ok(dates)
    }

    fn visit_weighted_urpd_dates(
        &self,
        cohort: &Cohort,
        weight: UrpdWeight,
        visit: impl FnMut(Date),
    ) -> Result<()> {
        let dir = self.urpd_dir(cohort)?;

        let all = Cohort::new(UTXO_ALL_NAME.id).expect("canonical cohort is valid");
        let weighted_cohort = if Self::urpd_aggregate_id(cohort).is_some() {
            cohort
        } else {
            &all
        };
        let weighted_dir = self.weighted_urpd_dir(weighted_cohort, weight)?;
        visit_intersection(dates_in_dir(&dir)?, dates_in_dir(&weighted_dir)?, visit);
        Ok(())
    }

    fn urpd_input_inner(
        &self,
        cohort: &Cohort,
        date: Date,
        weight: UrpdWeight,
    ) -> Result<(UrpdInput, f64)> {
        let raw_path = self.urpd_dir(cohort)?.join(date.to_string());

        if !raw_path.try_exists()? {
            return Err(Error::NotFound(format!(
                "No URPD for cohort '{cohort}' on {date}"
            )));
        }

        if weight == UrpdWeight::Raw {
            return Ok((self.read_urpd_input(cohort, date)?, 1.0));
        }

        if let Some(id) = Self::urpd_aggregate_id(cohort) {
            let path = self
                .weighted_urpd_dir(cohort, weight)?
                .join(date.to_string());
            if !path.try_exists()? {
                return Err(Error::NotFound(format!(
                    "No {weight}-weighted URPD for cohort '{cohort}' on {date}"
                )));
            }
            return Ok((
                UrpdInput::Raw(match weight {
                    UrpdWeight::Cointime => self.plugins().cointime.urpd.raw_bytes(id, date)?,
                    UrpdWeight::Coinflow => self.plugins().coinflow.urpd.raw_bytes(id, date)?,
                    UrpdWeight::Raw => unreachable!("raw URPDs are resolved above"),
                }),
                1.0,
            ));
        }

        let day = Day1::try_from(date)?;
        let scalar = Self::urpd_age_range_id(cohort)
            .and_then(|age| {
                let plugins = self.plugins();
                let supply = age
                    .select(&plugins.distribution.cohorts.supply.total.cohorts.utxo.age)
                    .sats
                    .day1
                    .collect_one(day)
                    .flatten()?;
                match weight {
                    UrpdWeight::Raw => Some(1.0),
                    UrpdWeight::Cointime => plugins.cointime.urpd_weight(age, day, supply),
                    UrpdWeight::Coinflow => plugins.coinflow.urpd_weight(age, day, supply),
                }
            })
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "No {weight} weight for cohort '{cohort}' on {date}"
                ))
            })?;
        Ok((self.read_urpd_input(cohort, date)?, scalar))
    }

    fn read_urpd_input(&self, cohort: &Cohort, date: Date) -> Result<UrpdInput> {
        let states_path = &self.plugins().distribution.states_path;
        if let Some(id) = Self::urpd_age_range_id(cohort) {
            return AgeRangeUrpds::read_one_bytes(states_path, id, date).map(UrpdInput::Raw);
        }
        if let Some(id) = Self::urpd_aggregate_id(cohort) {
            return AgeRangeUrpds::read_aggregate_encoded(states_path, id, date)
                .map(UrpdInput::Aggregate);
        }
        UrpdRaw::read_bytes(states_path, cohort, date).map(UrpdInput::Raw)
    }

    fn urpd_age_range_id(cohort: &Cohort) -> Option<AgeRangeId> {
        AgeRangeId::from_cohort_name(CohortContext::Utxo, cohort)
    }

    fn urpd_aggregate_id(cohort: &Cohort) -> Option<UTXOAggregateId> {
        UTXOAggregateId::from_cohort_name(cohort)
    }

    /// Capture one dated snapshot and its pricing inputs under publication protection.
    pub fn resolve_urpd_at(
        &self,
        cohort: &Cohort,
        date: Date,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let _guard = self.read_publication()?;
        self.resolve_urpd_inner(cohort, date, aggregation, weight)
    }

    /// Capture the latest snapshot and its pricing inputs under publication protection.
    /// Successful captures defer decoding to `ResolvedUrpd::build`; they are not
    /// evidence of a valid response until built.
    pub fn resolve_urpd_latest(
        &self,
        cohort: &Cohort,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let _guard = self.read_publication()?;
        let date = if weight == UrpdWeight::Raw {
            latest_date_in_dir(&self.urpd_dir(cohort)?)?
        } else {
            let mut latest = None;
            self.visit_weighted_urpd_dates(cohort, weight, |date| latest = Some(date))?;
            latest
        }
        .ok_or_else(|| {
            Error::NotFound(format!(
                "No {weight}-weighted URPD available for cohort '{cohort}'"
            ))
        })?;
        self.resolve_urpd_inner(cohort, date, aggregation, weight)
    }

    fn resolve_urpd_inner(
        &self,
        cohort: &Cohort,
        date: Date,
        aggregation: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<ResolvedUrpd> {
        let (input, scalar) = self.urpd_input_inner(cohort, date, weight)?;
        let close = Day1::try_from(date).and_then(|day| {
            self.plugins()
                .price
                .split
                .close
                .cents
                .day1
                .collect_one_flat(day)
                .ok_or_else(|| Error::NotFound(format!("No price data for {date}")))
        });
        let close = match close {
            Ok(close) => close,
            Err(error) => {
                // Preserve decoding-before-price error precedence without retaining
                // an error inside a successfully captured input.
                input.decode_entries()?;
                return Err(error);
            }
        };
        Ok(ResolvedUrpd {
            cohort: cohort.clone(),
            date,
            weight,
            aggregation,
            scalar,
            close,
            input,
        })
    }

    fn weighted_urpd_dir(&self, cohort: &Cohort, weight: UrpdWeight) -> Result<PathBuf> {
        let id = Self::urpd_aggregate_id(cohort)
            .ok_or_else(|| Error::NotFound(format!("No weighted URPD aggregate for '{cohort}'")))?;
        let dir = match weight {
            UrpdWeight::Cointime => self.plugins().cointime.urpd.dir(id),
            UrpdWeight::Coinflow => self.plugins().coinflow.urpd.dir(id),
            UrpdWeight::Raw => return self.urpd_dir(cohort),
        };
        if !dir.try_exists()? {
            return Err(Error::NotFound(format!(
                "No {weight}-weighted URPD available for cohort '{cohort}'"
            )));
        }
        Ok(dir)
    }
}

fn dates_in_dir(dir: &Path) -> Result<Vec<Date>> {
    let mut dates = Vec::new();
    visit_dates(dir, |date| dates.push(date))?;
    dates.sort_unstable();
    Ok(dates)
}

fn latest_date_in_dir(dir: &Path) -> Result<Option<Date>> {
    let mut latest = None;
    visit_dates(dir, |date| {
        latest = Some(latest.map_or(date, |old: Date| old.max(date)))
    })?;
    Ok(latest)
}

fn visit_dates(dir: &Path, mut visit: impl FnMut(Date)) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let Some(date) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse().ok())
        else {
            continue;
        };
        if entry.file_type()?.is_file() {
            visit(date);
        }
    }

    Ok(())
}

fn visit_intersection(left: Vec<Date>, right: Vec<Date>, mut visit: impl FnMut(Date)) {
    let mut left = left.into_iter().peekable();
    let mut right = right.into_iter().peekable();
    while let (Some(&a), Some(&b)) = (left.peek(), right.peek()) {
        match a.cmp(&b) {
            Ordering::Less => {
                left.next();
            }
            Ordering::Greater => {
                right.next();
            }
            Ordering::Equal => {
                visit(a);
                left.next();
                right.next();
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/impl/urpd.rs"]
mod tests;

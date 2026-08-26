use std::{
    cmp::Ordering,
    fs,
    path::{Path, PathBuf},
};

use bitview_cohort::{AgeRangeId, CohortContext, UTXO_ALL_NAME, UTXOAggregateId};
use bitview_plugin::{Plugin, PluginReadGuard};
use bitview_plugin_distribution::AgeRangeUrpds;
use brk_error::{Error, Result};
use brk_types::{Cohort, Date, Day1, Urpd, UrpdAggregation, UrpdRaw, UrpdWeight};
use vecdb::{ColumnId, ReadableOptionVec};

use crate::Query;

impl Query {
    fn urpd_read_guard(&self) -> Result<PluginReadGuard> {
        self.read_plugins(&[
            self.plugins().distribution as &dyn Plugin,
            self.plugins().bedrock as &dyn Plugin,
        ])
    }

    /// Available cohorts for URPD.
    pub fn urpd_cohorts(&self) -> brk_error::Result<Vec<Cohort>> {
        let _guard = self.urpd_read_guard()?;
        self.urpd_cohorts_inner()
    }

    fn urpd_cohorts_inner(&self) -> brk_error::Result<Vec<Cohort>> {
        let states_path = &self.plugins().distribution.states_path;
        let age_range_dir = AgeRangeUrpds::dir(states_path);

        let mut cohorts: Vec<Cohort> = fs::read_dir(states_path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.path() == age_range_dir || !entry.file_type().ok()?.is_dir() {
                    return None;
                }
                let cohort = Cohort::new(entry.file_name().into_string().ok()?)?;
                if Self::urpd_age_range_id(&cohort).is_some()
                    || Self::urpd_aggregate_id(&cohort).is_some()
                {
                    return None;
                }
                Some(cohort)
            })
            .collect();

        if age_range_dir.exists() {
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

    fn urpd_dir(&self, cohort: &Cohort) -> brk_error::Result<PathBuf> {
        let states_path = &self.plugins().distribution.states_path;
        let is_age_range = Self::urpd_age_range_id(cohort).is_some();
        let is_aggregate = Self::urpd_aggregate_id(cohort).is_some();
        let dir = if is_age_range || is_aggregate {
            AgeRangeUrpds::dir(states_path)
        } else {
            UrpdRaw::dir(states_path, cohort)
        };

        if !dir.exists() {
            let valid = self
                .urpd_cohorts_inner()
                .unwrap_or_default()
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(Error::NotFound(format!(
                "Unknown cohort '{cohort}'. Available: {valid}"
            )));
        }

        Ok(dir)
    }

    /// Available dates for a cohort.
    pub fn urpd_dates(&self, cohort: &Cohort) -> brk_error::Result<Vec<Date>> {
        self.urpd_dates_with_weight(cohort, UrpdWeight::Raw)
    }

    /// Available dates for a cohort and weighting.
    pub fn urpd_dates_with_weight(
        &self,
        cohort: &Cohort,
        weight: UrpdWeight,
    ) -> brk_error::Result<Vec<Date>> {
        let _guard = self.urpd_read_guard()?;
        self.urpd_dates_with_weight_inner(cohort, weight)
    }

    fn urpd_dates_with_weight_inner(
        &self,
        cohort: &Cohort,
        weight: UrpdWeight,
    ) -> brk_error::Result<Vec<Date>> {
        let dir = self.urpd_dir(cohort)?;

        if weight == UrpdWeight::Raw {
            return dates_in_dir(&dir);
        }

        let all = Cohort::new(UTXO_ALL_NAME.id).expect("canonical cohort is valid");
        let weighted_cohort = if Self::urpd_aggregate_id(cohort).is_some() {
            cohort
        } else {
            &all
        };
        let weighted_dir = self.weighted_urpd_dir(weighted_cohort, weight)?;
        Ok(intersect_dates(
            dates_in_dir(&dir)?,
            dates_in_dir(&weighted_dir)?,
        ))
    }

    /// Raw URPD data for a cohort on a specific date.
    pub fn urpd_raw(&self, cohort: &Cohort, date: Date) -> brk_error::Result<UrpdRaw> {
        self.urpd_raw_with_weight(cohort, date, UrpdWeight::Raw)
    }

    /// Raw URPD data with an optional Bedrock weighting.
    pub fn urpd_raw_with_weight(
        &self,
        cohort: &Cohort,
        date: Date,
        weight: UrpdWeight,
    ) -> brk_error::Result<UrpdRaw> {
        let _guard = self.urpd_read_guard()?;
        self.urpd_raw_with_weight_inner(cohort, date, weight)
    }

    fn urpd_raw_with_weight_inner(
        &self,
        cohort: &Cohort,
        date: Date,
        weight: UrpdWeight,
    ) -> brk_error::Result<UrpdRaw> {
        let raw_path = self.urpd_dir(cohort)?.join(date.to_string());

        if !raw_path.exists() {
            return Err(Error::NotFound(format!(
                "No URPD for cohort '{cohort}' on {date}"
            )));
        }

        if weight == UrpdWeight::Raw {
            return self.read_raw_urpd(cohort, date);
        }

        if Self::urpd_aggregate_id(cohort).is_some() {
            let path = self
                .weighted_urpd_dir(cohort, weight)?
                .join(date.to_string());
            if !path.exists() {
                return Err(Error::NotFound(format!(
                    "No {weight}-weighted URPD for cohort '{cohort}' on {date}"
                )));
            }
            return self.plugins().bedrock.urpd_raw(weight, cohort, date);
        }

        let day = Day1::try_from(date)?;
        let scalar = self
            .plugins()
            .bedrock
            .urpd_weight(
                self.plugins().distribution,
                self.plugins().cointime,
                self.plugins().coinflow,
                cohort,
                day,
                weight,
            )
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "No {weight} weight for cohort '{cohort}' on {date}"
                ))
            })?;
        Ok(self.read_raw_urpd(cohort, date)?.apply_weight(scalar))
    }

    fn read_raw_urpd(&self, cohort: &Cohort, date: Date) -> Result<UrpdRaw> {
        let states_path = &self.plugins().distribution.states_path;
        if let Some(id) = Self::urpd_age_range_id(cohort) {
            return AgeRangeUrpds::read_one(states_path, id, date);
        }
        if let Some(id) = Self::urpd_aggregate_id(cohort) {
            return AgeRangeUrpds::read_aggregate(states_path, id, date);
        }
        UrpdRaw::read(states_path, cohort, date)
    }

    fn urpd_age_range_id(cohort: &Cohort) -> Option<AgeRangeId> {
        AgeRangeId::from_cohort_name(CohortContext::Utxo, cohort)
    }

    fn urpd_aggregate_id(cohort: &Cohort) -> Option<UTXOAggregateId> {
        UTXOAggregateId::from_cohort_name(cohort)
    }

    /// URPD for a cohort on a specific date.
    pub fn urpd_at(
        &self,
        cohort: &Cohort,
        date: Date,
        agg: UrpdAggregation,
    ) -> brk_error::Result<Urpd> {
        self.urpd_at_with_weight(cohort, date, agg, UrpdWeight::Raw)
    }

    /// URPD for a cohort on a specific date and weighting.
    pub fn urpd_at_with_weight(
        &self,
        cohort: &Cohort,
        date: Date,
        agg: UrpdAggregation,
        weight: UrpdWeight,
    ) -> brk_error::Result<Urpd> {
        let _guard = self.urpd_read_guard()?;
        self.urpd_at_with_weight_inner(cohort, date, agg, weight)
    }

    fn urpd_at_with_weight_inner(
        &self,
        cohort: &Cohort,
        date: Date,
        agg: UrpdAggregation,
        weight: UrpdWeight,
    ) -> brk_error::Result<Urpd> {
        let raw = self.urpd_raw_with_weight_inner(cohort, date, weight)?;
        let day1 = Day1::try_from(date)?;
        let close = self
            .plugins()
            .price
            .split
            .close
            .cents
            .day1
            .collect_one_flat(day1)
            .ok_or_else(|| Error::NotFound(format!("No price data for {date}")))?;
        Ok(Urpd::build(cohort.clone(), date, weight, close, &raw, agg))
    }

    /// URPD for the most recently available date in a cohort.
    pub fn urpd_latest(&self, cohort: &Cohort, agg: UrpdAggregation) -> brk_error::Result<Urpd> {
        self.urpd_latest_with_weight(cohort, agg, UrpdWeight::Raw)
    }

    /// Most recent URPD for a cohort and weighting.
    pub fn urpd_latest_with_weight(
        &self,
        cohort: &Cohort,
        agg: UrpdAggregation,
        weight: UrpdWeight,
    ) -> brk_error::Result<Urpd> {
        let _guard = self.urpd_read_guard()?;
        let dates = self.urpd_dates_with_weight_inner(cohort, weight)?;
        let date = *dates.last().ok_or_else(|| {
            Error::NotFound(format!(
                "No {weight}-weighted URPD available for cohort '{cohort}'"
            ))
        })?;
        self.urpd_at_with_weight_inner(cohort, date, agg, weight)
    }

    fn weighted_urpd_dir(&self, cohort: &Cohort, weight: UrpdWeight) -> brk_error::Result<PathBuf> {
        let dir = self.plugins().bedrock.urpd_dir(weight, cohort);
        if !dir.exists() {
            return Err(Error::NotFound(format!(
                "No {weight}-weighted URPD available for cohort '{cohort}'"
            )));
        }
        Ok(dir)
    }
}

fn dates_in_dir(dir: &Path) -> brk_error::Result<Vec<Date>> {
    let mut dates: Vec<Date> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok()?.file_name().to_str()?.parse().ok())
        .collect();

    dates.sort_unstable();
    Ok(dates)
}

fn intersect_dates(left: Vec<Date>, right: Vec<Date>) -> Vec<Date> {
    let mut left = left.into_iter().peekable();
    let mut right = right.into_iter().peekable();
    let mut intersection = Vec::new();

    while let (Some(&a), Some(&b)) = (left.peek(), right.peek()) {
        match a.cmp(&b) {
            Ordering::Less => {
                left.next();
            }
            Ordering::Greater => {
                right.next();
            }
            Ordering::Equal => {
                intersection.push(a);
                left.next();
                right.next();
            }
        }
    }

    intersection
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_intersection_stays_sorted() {
        let a = Date::new(2026, 8, 1);
        let b = Date::new(2026, 8, 2);
        let c = Date::new(2026, 8, 3);

        assert_eq!(intersect_dates(vec![a, b, c], vec![a, c]), vec![a, c]);
    }
}

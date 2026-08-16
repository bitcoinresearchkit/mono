use std::{
    fs,
    path::{Path, PathBuf},
};

use brk_cohort::{UTXO_AGGREGATE_NAMES, UTXO_ALL_NAME};
use brk_error::{Error, Result};
use brk_plugin::PluginReadGuard;
use brk_types::{Cohort, Date, Day1, Urpd, UrpdAggregation, UrpdRaw, UrpdWeight};
use vecdb::ReadableOptionVec;

use crate::Query;

impl Query {
    fn urpd_read_guard(&self) -> PluginReadGuard {
        PluginReadGuard::acquire(&[
            self.computer().distribution.as_ref(),
            self.computer().models.as_ref(),
        ])
    }

    /// Available cohorts for URPD.
    pub fn urpd_cohorts(&self) -> Result<Vec<Cohort>> {
        let _guard = self.urpd_read_guard();
        self.urpd_cohorts_inner()
    }

    fn urpd_cohorts_inner(&self) -> Result<Vec<Cohort>> {
        let states_path = &self.computer().distribution.states_path;

        let mut cohorts: Vec<Cohort> = fs::read_dir(states_path)?
            .filter_map(|entry| {
                let name = entry.ok()?.file_name().into_string().ok()?;
                if !UrpdRaw::dir(states_path, &name).exists() {
                    return None;
                }
                Cohort::new(name)
            })
            .collect();

        cohorts.sort_unstable();

        Ok(cohorts)
    }

    pub(crate) fn urpd_dir(&self, cohort: &Cohort) -> Result<PathBuf> {
        let dir = UrpdRaw::dir(&self.computer().distribution.states_path, cohort);

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
    pub fn urpd_dates(&self, cohort: &Cohort) -> Result<Vec<Date>> {
        self.urpd_dates_with_weight(cohort, UrpdWeight::Raw)
    }

    /// Available dates for a cohort and weighting.
    pub fn urpd_dates_with_weight(&self, cohort: &Cohort, weight: UrpdWeight) -> Result<Vec<Date>> {
        let _guard = self.urpd_read_guard();
        self.urpd_dates_with_weight_inner(cohort, weight)
    }

    fn urpd_dates_with_weight_inner(
        &self,
        cohort: &Cohort,
        weight: UrpdWeight,
    ) -> Result<Vec<Date>> {
        let dir = self.urpd_dir(cohort)?;

        if weight == UrpdWeight::Raw {
            return dates_in_dir(&dir);
        }

        let all = Cohort::new(UTXO_ALL_NAME.id).expect("canonical cohort is valid");
        let weighted_cohort = if is_aggregate_cohort(cohort) {
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
    pub fn urpd_raw(&self, cohort: &Cohort, date: Date) -> Result<UrpdRaw> {
        self.urpd_raw_with_weight(cohort, date, UrpdWeight::Raw)
    }

    /// Raw URPD data with an optional Bedrock weighting.
    pub fn urpd_raw_with_weight(
        &self,
        cohort: &Cohort,
        date: Date,
        weight: UrpdWeight,
    ) -> Result<UrpdRaw> {
        let _guard = self.urpd_read_guard();
        self.urpd_raw_with_weight_inner(cohort, date, weight)
    }

    fn urpd_raw_with_weight_inner(
        &self,
        cohort: &Cohort,
        date: Date,
        weight: UrpdWeight,
    ) -> Result<UrpdRaw> {
        let raw_path = self.urpd_dir(cohort)?.join(date.to_string());

        if !raw_path.exists() {
            return Err(Error::NotFound(format!(
                "No URPD for cohort '{cohort}' on {date}"
            )));
        }

        if weight == UrpdWeight::Raw {
            return UrpdRaw::read(&self.computer().distribution.states_path, cohort, date);
        }

        if is_aggregate_cohort(cohort) {
            let path = self
                .weighted_urpd_dir(cohort, weight)?
                .join(date.to_string());
            if !path.exists() {
                return Err(Error::NotFound(format!(
                    "No {weight}-weighted URPD for cohort '{cohort}' on {date}"
                )));
            }
            return self.computer().bedrock_urpd_raw(weight, cohort, date);
        }

        let day = Day1::try_from(date)?;
        let scalar = self
            .computer()
            .bedrock_urpd_weight(cohort, day, weight)
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "No {weight} weight for cohort '{cohort}' on {date}"
                ))
            })?;
        Ok(
            UrpdRaw::read(&self.computer().distribution.states_path, cohort, date)?
                .apply_weight(scalar),
        )
    }

    /// URPD for a cohort on a specific date.
    pub fn urpd_at(&self, cohort: &Cohort, date: Date, agg: UrpdAggregation) -> Result<Urpd> {
        self.urpd_at_with_weight(cohort, date, agg, UrpdWeight::Raw)
    }

    /// URPD for a cohort on a specific date and weighting.
    pub fn urpd_at_with_weight(
        &self,
        cohort: &Cohort,
        date: Date,
        agg: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<Urpd> {
        let _guard = self.urpd_read_guard();
        self.urpd_at_with_weight_inner(cohort, date, agg, weight)
    }

    fn urpd_at_with_weight_inner(
        &self,
        cohort: &Cohort,
        date: Date,
        agg: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<Urpd> {
        let raw = self.urpd_raw_with_weight_inner(cohort, date, weight)?;
        let day1 = Day1::try_from(date)?;
        let close = self
            .computer()
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
    pub fn urpd_latest(&self, cohort: &Cohort, agg: UrpdAggregation) -> Result<Urpd> {
        self.urpd_latest_with_weight(cohort, agg, UrpdWeight::Raw)
    }

    /// Most recent URPD for a cohort and weighting.
    pub fn urpd_latest_with_weight(
        &self,
        cohort: &Cohort,
        agg: UrpdAggregation,
        weight: UrpdWeight,
    ) -> Result<Urpd> {
        let _guard = self.urpd_read_guard();
        let dates = self.urpd_dates_with_weight_inner(cohort, weight)?;
        let date = *dates.last().ok_or_else(|| {
            Error::NotFound(format!(
                "No {weight}-weighted URPD available for cohort '{cohort}'"
            ))
        })?;
        self.urpd_at_with_weight_inner(cohort, date, agg, weight)
    }

    fn weighted_urpd_dir(&self, cohort: &Cohort, weight: UrpdWeight) -> Result<PathBuf> {
        let dir = self.computer().bedrock_urpd_dir(weight, cohort);
        if !dir.exists() {
            return Err(Error::NotFound(format!(
                "No {weight}-weighted URPD available for cohort '{cohort}'"
            )));
        }
        Ok(dir)
    }
}

fn is_aggregate_cohort(cohort: &Cohort) -> bool {
    UTXO_AGGREGATE_NAMES.iter().any(|name| name.id == &**cohort)
}

fn dates_in_dir(dir: &Path) -> Result<Vec<Date>> {
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
            std::cmp::Ordering::Less => {
                left.next();
            }
            std::cmp::Ordering::Greater => {
                right.next();
            }
            std::cmp::Ordering::Equal => {
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

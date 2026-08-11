mod compute;
mod import;
mod levels;
mod mode_vecs;
mod modes;
mod percentiles;
mod price;
mod price_bands;
mod vecs;
mod weighted;

pub(super) use levels::Levels;
pub(super) use mode_vecs::ModeVecs;
pub(super) use modes::Modes;
pub(super) use percentiles::Percentiles;
pub(super) use price_bands::PriceBands;
pub(super) use weighted::WeightedModes;

use std::path::PathBuf;

use brk_cohort::{AgeRangeId, UTXO_ALL_NAME};
use brk_error::Result;
use brk_types::{Cohort, Date, Day1, UrpdRaw, UrpdWeight};
use vecdb::{ColumnId, ReadableVec, StorageMode};

use self::compute::resolve_age_value;
use crate::Computer;

pub use vecs::Vecs;

pub(crate) fn weighted_urpd_name(weight: UrpdWeight, cohort: &str) -> String {
    debug_assert!(weight.is_weighted());
    if cohort == UTXO_ALL_NAME.id {
        format!("bedrock_{}", weight.as_str())
    } else {
        format!("bedrock_{}_{cohort}", weight.as_str())
    }
}

impl<M: StorageMode> Computer<M> {
    /// Directory containing a persisted aggregate Bedrock-weighted URPD.
    pub fn bedrock_urpd_dir(&self, weight: UrpdWeight, cohort: &Cohort) -> PathBuf {
        UrpdRaw::dir(
            &self.models.bedrock.states_path,
            &weighted_urpd_name(weight, cohort),
        )
    }

    /// Read a persisted aggregate Bedrock-weighted URPD.
    pub fn bedrock_urpd_raw(
        &self,
        weight: UrpdWeight,
        cohort: &Cohort,
        date: Date,
    ) -> Result<UrpdRaw> {
        UrpdRaw::read(
            &self.models.bedrock.states_path,
            &weighted_urpd_name(weight, cohort),
            date,
        )
    }

    /// Resolve one age-range cohort's scalar Bedrock weight for a day.
    pub fn bedrock_urpd_weight(
        &self,
        cohort: &Cohort,
        day: Day1,
        weight: UrpdWeight,
    ) -> Option<f64> {
        let cohort_id = cohort.strip_prefix("utxos_")?;
        let age = AgeRangeId::ALL
            .iter()
            .copied()
            .find(|age| age.name().id == cohort_id)?;

        if weight == UrpdWeight::Raw {
            return Some(1.0);
        }

        let supplies = &self.distribution.cohorts.supply.total.cohorts.age.range;
        let supply = age.select(supplies).sats.day1.collect_one(day).flatten()?;

        match weight {
            UrpdWeight::Raw => Some(1.0),
            UrpdWeight::Cointime => {
                let cohort = age.select(&self.frameworks.cointime.age_range.activity.wakefulness);
                resolve_age_value(cohort.day1.collect_one(day).flatten(), supply)
                    .map(|value| value.clamp(0.0, 1.0))
            }
            UrpdWeight::Coinflow => {
                let cohort = age.select(
                    &self
                        .frameworks
                        .coinflow
                        .age_range
                        .spending_exposure
                        .mobility,
                );
                resolve_age_value(cohort.day1.0.collect_one(day).flatten(), supply)
                    .map(|value| value.clamp(0.0, 1.0))
            }
        }
    }
}

macro_rules! impl_named_row_formattable {
    ($row:ident { $($field:ident),+ $(,)? }) => {
        impl<T: Formattable> Formattable for $row<T> {
            fn write_to(&self, output: &mut Vec<u8>) {
                output.push(b'{');
                let mut first = true;
                $(
                    if !first {
                        output.push(b',');
                    }
                    first = false;
                    output.extend_from_slice(concat!("\"", stringify!($field), "\":").as_bytes());
                    self.$field.fmt_json(output);
                )+
                let _ = first;
                output.push(b'}');
            }

            fn fmt_csv(&self, output: &mut String) -> fmt::Result {
                let mut json = Vec::new();
                self.write_to(&mut json);
                let json = str::from_utf8(&json).map_err(|_| fmt::Error)?;

                output.push('"');
                for character in json.chars() {
                    if character == '"' {
                        output.push('"');
                    }
                    output.push(character);
                }
                output.push('"');
                Ok(())
            }
        }
    };
}

mod calibration;
mod compute;
mod day_result;
mod day_urpds;
mod import;
mod level_id;
mod levels;
mod loss_percentile_id;
mod mode_id;
mod mode_result;
mod mode_vecs;
mod modes;
mod percentiles;
mod price;
mod price_band_id;
mod price_bands;
mod vecs;
mod weighted;
mod weighted_pair;

use calibration::{Calibration, Thresholds};
use day_result::DayResult;
use day_urpds::{DayUrpds, ModeWeights};
use level_id::{LEVEL_COUNT, LEVEL_IDS, LevelId};
use levels::Levels;
use loss_percentile_id::LossPercentileId;
use mode_id::{MODE_COUNT, ModeId};
use mode_result::ModeResult;
use mode_vecs::ModeVecs;
use modes::Modes;
use percentiles::Percentiles;
use price_band_id::PriceBandId;
use price_bands::PriceBands;
use weighted::{WeightedModeId, WeightedModes};
use weighted_pair::WeightedPair;

use std::path::PathBuf;

use brk_cohort::AgeRangeId;
use brk_error::Result;
use brk_types::{Cohort, Date, Day1, UrpdRaw, UrpdWeight};
use vecdb::{ColumnId, ReadableVec, StorageMode};

use crate::Computer;

pub use vecs::Vecs;

impl<M: StorageMode> Computer<M> {
    /// Directory containing a persisted aggregate Bedrock-weighted URPD.
    pub fn bedrock_urpd_dir(&self, weight: UrpdWeight, cohort: &Cohort) -> PathBuf {
        UrpdRaw::dir(
            &self.models.bedrock.states_path,
            &DayUrpds::weighted_name(weight, cohort),
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
            &DayUrpds::weighted_name(weight, cohort),
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
                Vecs::<M>::resolve_age_value(cohort.day1.collect_one(day).flatten(), supply)
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
                Vecs::<M>::resolve_age_value(cohort.day1.0.collect_one(day).flatten(), supply)
                    .map(|value| value.clamp(0.0, 1.0))
            }
        }
    }
}

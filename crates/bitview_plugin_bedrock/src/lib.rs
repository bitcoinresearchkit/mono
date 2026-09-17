macro_rules! impl_named_row_formattable {
    ($collection:ident { $($field:ident),+ $(,)? }) => {
        impl<T: Formattable> Formattable for $collection<T> {
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

mod age_cutoffs;
mod age_density_vecs;
mod age_price_bounds;
mod age_price_bounds_vecs;
mod calibration;
mod capitalized_price;
mod capitalized_price_vecs;
mod cost_basis_data;
mod cost_basis_vecs;
mod day_result;
mod day_urpds;
mod density_bands;
mod dependencies;
mod has;
mod level_id;
mod levels;
mod loss_percentile_id;
mod mode_id;
mod mode_result;
mod mode_vecs;
mod mode_weights;
mod modes;
mod percentiles;
mod price_band_id;
mod price_bands;
mod price_bounds;
mod supply_density;
mod supply_density_vecs;
mod thresholds;
mod vecs;
mod weighted;
mod weighted_pair;
mod weighted_urpd_names;

use age_cutoffs::AgeCutoffs;
use age_density_vecs::AgeDensityVecs;
use age_price_bounds_vecs::AgePriceBoundsVecs;
use bitview_vecs::DailyPercentilesVecs;
use calibration::Calibration;
use capitalized_price_vecs::CapitalizedPriceVecs;
use cost_basis_data::CostBasisData;
use cost_basis_vecs::CostBasisVecs;
use day_result::DayResult;
use day_urpds::DayUrpds;
use density_bands::DensityBands;
pub use dependencies::Dependencies;
pub use has::HasBedrock;
use level_id::{LEVEL_COUNT, LEVEL_IDS, LevelId};
use levels::Levels;
use loss_percentile_id::LossPercentileId;
use mode_id::{MODE_COUNT, ModeId};
use mode_result::ModeResult;
use mode_vecs::ModeVecs;
use mode_weights::ModeWeights;
use modes::Modes;
use percentiles::Percentiles;
use price_band_id::PriceBandId;
use price_bands::PriceBands;
use price_bounds::PriceBounds;
use supply_density::SupplyDensity;
use supply_density_vecs::SupplyDensityVecs;
use thresholds::Thresholds;
use weighted::{WeightedModeId, WeightedModes};
use weighted_pair::WeightedPair;
use weighted_urpd_names::WeightedUrpdNames;

use bitview_plugin::{PluginId, PluginStorage};
use brk_types::Version;

pub use vecs::Vecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("bedrock"), Version::new(14));
pub const ID: PluginId = STORAGE.id();

const WRITE_INTERVAL_DAYS: usize = 100;

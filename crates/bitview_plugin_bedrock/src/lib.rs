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
mod cost_basis_vecs;
mod daily_percentiles_vecs;
mod day_result;
mod day_urpds;
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
mod price;
mod price_band_id;
mod price_bands;
mod thresholds;
mod vecs;
mod weighted;
mod weighted_pair;
mod weighted_urpd_names;

use calibration::Calibration;
use cost_basis_vecs::CostBasisVecs;
use daily_percentiles_vecs::DailyPercentilesVecs;
use day_result::DayResult;
use day_urpds::DayUrpds;
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
use thresholds::Thresholds;
use weighted::{WeightedModeId, WeightedModes};
use weighted_pair::WeightedPair;
use weighted_urpd_names::WeightedUrpdNames;

use bitview_plugin::{PluginId, PluginStorage};
use brk_types::Version;

pub use vecs::Vecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("bedrock"), Version::new(14));
pub const ID: PluginId = STORAGE.id();

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

mod block_result;
mod calibration;
mod cumulative_bucket;
mod dependencies;
mod level_id;
mod levels;
mod loss_percentile_id;
mod mode_id;
mod mode_result;
mod mode_vecs;
mod modes;
mod percentiles;
mod price_band_id;
mod price_bands;
mod thresholds;
mod vecs;
mod weighted;

use block_result::BlockResult;
use calibration::Calibration;
use cumulative_bucket::CumulativeBucket;
pub use dependencies::Dependencies;
use level_id::{LEVEL_COUNT, LevelId};
use levels::Levels;
use loss_percentile_id::LossPercentileId;
use mode_id::{MODE_COUNT, ModeId};
use mode_result::ModeResult;
use mode_vecs::ModeVecs;
use modes::Modes;
use percentiles::Percentiles;
use price_band_id::PriceBandId;
use price_bands::PriceBands;
use thresholds::Thresholds;
use weighted::{WeightedModeId, WeightedModes};

use bitview_plugin::{PluginId, PluginStorage};
use brk_types::Version;

pub use vecs::Vecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("bedrock"), Version::new(15));
pub const ID: PluginId = STORAGE.id();

const WRITE_INTERVAL_BLOCKS: usize = 10_000;

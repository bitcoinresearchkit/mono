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
mod compute;
mod cumulative_bucket;
mod dependencies;
mod import;
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
mod weighted;

pub use dependencies::Dependencies;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use bitview_urpd::Replay;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use block_result::BlockResult;
use calibration::Calibration;
use cumulative_bucket::CumulativeBucket;
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

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("bedrock"), Version::new(15));
pub const ID: PluginId = STORAGE.id();
const WRITE_INTERVAL_BLOCKS: usize = 10_000;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    #[traversable(skip)]
    calibration: M::WriteOnly<Option<Calibration>>,
    #[traversable(skip)]
    replay: M::WriteOnly<Replay>,
    #[traversable(skip)]
    scratch: M::WriteOnly<Vec<CumulativeBucket>>,

    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    modes: Modes<ModeVecs<M>>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

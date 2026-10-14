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

pub use dependencies::Dependencies;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use bitview_urpd::Replay;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use block_result::BlockResult;
use calibration::Calibration;
use cumulative_bucket::CumulativeBucket;
use level_id::LevelId;
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

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("bedrock"), Version::new(17));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    #[traversable(skip)]
    calibration: M::WriteOnly<Option<Calibration>>,
    #[traversable(skip)]
    replay: M::WriteOnly<Replay>,

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

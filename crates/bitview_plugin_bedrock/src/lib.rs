#[macro_use]
mod collection;

mod block_result;
mod calibration;
mod compute;
mod cumulative_bucket;
mod dependencies;
mod entry_supply;
mod entry_vecs;
mod floor_bands;
mod import;
mod levels;
mod mode_result;
mod mode_vecs;
mod modes;
mod percentiles;
mod price_band_id;
mod price_bands;

pub use dependencies::Dependencies;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use bitview_urpd::Replay;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use block_result::BlockResult;
use calibration::Calibration;
use cumulative_bucket::CumulativeBucket;
use entry_supply::{EntrySupply, Holding};
use entry_vecs::EntryVecs;
use floor_bands::{FloorBandId, FloorBands};
use levels::{LevelId, Levels};
use mode_result::ModeResult;
use mode_vecs::ModeVecs;
use modes::{MODE_COUNT, ModeId, Modes, Thresholds};
use percentiles::{LossPercentileId, Percentiles};
use price_band_id::PriceBandId;
use price_bands::PriceBands;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("bedrock"), Version::new(18));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    calibration: M::WriteOnly<Option<Calibration>>,
    replay: M::WriteOnly<Replay>,
    entry_supply: M::WriteOnly<EntrySupply>,

    #[traversable(flatten)]
    #[traversable(lead = "Bedrock: price floors where holders' losses would reach their historic extremes.")]
    modes: Modes<ModeVecs<M>>,
}

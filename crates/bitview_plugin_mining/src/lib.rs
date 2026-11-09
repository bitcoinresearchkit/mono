mod compute;
mod dependencies;
mod has;
mod hash;
mod import;
mod rewards;

pub use dependencies::Dependencies;
pub use has::HasMining;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::Hashrate;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, shapes::Windows};
use brk_types::{Height, Version};
use vecdb::{Database, Rw, StorageMode};

use hash::Vecs as HashVecs;
use rewards::Vecs as RewardsVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("mining"), Version::new(11));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    pub rewards: RewardsVecs<M>,
    #[traversable(flatten)]
    pub hash: HashVecs<M>,
}

impl<M: StorageMode> Vecs<M> {
    /// Network hash-rate estimates for 24 hours and their 1-week, 1-month and 1-year averages.
    pub fn network_hashrate_windows(&self) -> Windows<&CachedSeries<Height, Hashrate, M>> {
        let rate = &self.hash.hashrate;
        Windows {
            _24h: &rate.value.height,
            _1w: &rate.sma._1w.height,
            _1m: &rate.sma._1m.height,
            _1y: &rate.sma._1y.height,
        }
    }
}

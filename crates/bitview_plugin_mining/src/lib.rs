mod dependencies;
mod has;
mod hashrate;
mod rewards;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

pub use dependencies::Dependencies;
pub use has::HasMining;
use hashrate::Vecs as HashrateVecs;
use rewards::Vecs as RewardsVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("mining"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    pub rewards: RewardsVecs<M>,
    pub hashrate: HashrateVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

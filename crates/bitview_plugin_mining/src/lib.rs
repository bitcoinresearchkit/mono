mod dependencies;
mod hashrate;
mod rewards;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginId};
use bitview_traversable::Traversable;
use vecdb::{Database, Rw, StorageMode};

pub use dependencies::Dependencies;
use hashrate::Vecs as HashrateVecs;
use rewards::Vecs as RewardsVecs;

pub const ID: PluginId = PluginId::new("mining");
const DB_NAME: &str = ID.as_str();

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
    Self: Send + Sync,
{
    fn id(&self) -> PluginId {
        ID
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

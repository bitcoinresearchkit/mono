pub mod hashrate;
pub mod rewards;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginId};
use brk_traversable::Traversable;
use vecdb::{Database, Rw, StorageMode};

pub use hashrate::Vecs as HashrateVecs;
pub use rewards::Vecs as RewardsVecs;

pub const DB_NAME: &str = "mining";

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub(crate) db: Database,

    pub rewards: RewardsVecs<M>,
    pub hashrate: HashrateVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> PluginId {
        PluginId::new(DB_NAME)
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

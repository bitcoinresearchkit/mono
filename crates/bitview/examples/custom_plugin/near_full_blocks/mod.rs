mod compute;
mod dependencies;
mod has;
mod import;

pub use dependencies::Dependencies;
pub use has::HasNearFullBlocks;

use bitview_plugin::{Plugin, PluginGate, PluginId};
use bitview_traversable::Traversable;
use brk_types::{Height, StoredU64};
use vecdb::{Database, EagerVec, PcoVec, Rw, StorageMode};

pub const ID: PluginId = PluginId::new("near_full_blocks");

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    /// Consecutive blocks whose weight is at least 90% of Bitcoin's consensus
    /// maximum. Resets to zero whenever a block falls below that threshold.
    pub streak: M::Stored<EagerVec<PcoVec<Height, StoredU64>>>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn id(&self) -> PluginId {
        ID
    }

    fn gate(&self) -> &PluginGate {
        &self.gate
    }
}

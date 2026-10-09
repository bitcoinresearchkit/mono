#![doc = include_str!("../README.md")]

mod compute;
mod dependencies;
mod has;
mod import;
mod live;
mod metrics;

pub use dependencies::Dependencies;
pub use has::HasEntry;

use bitview_cohort::ByEntry;
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use live::LiveState;
use metrics::CohortMetrics;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("entry"), Version::ONE);
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    cohorts: ByEntry<CohortMetrics<M>>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

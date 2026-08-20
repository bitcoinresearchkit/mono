mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginStorage};
use bitview_traversable::Traversable;
use vecdb::{Database, Rw, StorageMode};

use super::{AgeRangeVecs, AggregateSources, AggregateVecs};
use crate::STORAGE;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    pub age_range: AgeRangeVecs<M>,
    #[traversable(flatten)]
    pub all: AggregateVecs,
    pub sth: AggregateVecs,
    pub lth: AggregateVecs,
    pub aggregate_sources: AggregateSources<M>,
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

#![doc = include_str!("../README.md")]

mod bucket;
mod compute;
mod dependencies;
mod import;
mod live;
mod metrics;
mod terms;

pub use dependencies::Dependencies;
pub use metrics::Metrics;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use live::LiveState;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("profitability"), Version::new(5));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    #[traversable(flatten)]
    metrics: Box<Metrics<M>>,
}

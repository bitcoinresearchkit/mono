#![doc = include_str!("../README.md")]

mod bucket;
mod compute;
mod dependencies;
mod has;
mod import;
mod live;
mod metrics;

pub use dependencies::Dependencies;
pub use has::HasDistributionProfitability;
pub use metrics::Metrics;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use live::LiveState;

const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("distribution_profitability"), Version::TWO);

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    #[traversable(wrap = "cohorts", rename = "profitability")]
    metrics: Box<Metrics<M>>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

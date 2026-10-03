#![doc = include_str!("../README.md")]

mod activity;
mod adjusted_sopr;
mod columns;
mod compute;
mod cost_basis;
mod data;
mod density_sources;
mod dependencies;
mod has;
mod live;
mod metrics;
mod outputs;
mod ratios;
mod realized;
mod relative;
mod sources;
mod supply;
mod unrealized;
mod unrealized_data;
mod vecs;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
pub use dependencies::Dependencies;
pub use has::HasDistributionAggregated;
pub use metrics::Metrics;
use vecdb::StorageMode;
pub use vecs::Vecs;
const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("distribution_aggregated"), Version::ONE);
pub const ID: PluginId = STORAGE.id();
impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

mod dependencies;
mod live;
pub use dependencies::Dependencies;
mod has;
pub use has::HasDistributionAge;
mod compute;
mod metrics;
mod state;
mod vecs;
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_oracle::VERSION as ORACLE_VERSION;
use brk_types::Version;
use vecdb::StorageMode;
pub use vecs::Vecs;
const STORAGE: PluginStorage = PluginStorage::new(
    PluginId::new("distribution_age"),
    Version::new(41 + ORACLE_VERSION),
);
pub const ID: PluginId = STORAGE.id();

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

#[cfg(test)]
mod test_cache;

mod accounting_sources;
mod all_chain_sources;
pub use accounting_sources::AccountingSources;
pub use all_chain_sources::AllChainSources;
pub use bitview_plugin_distribution_common::metrics::RealizedTotals;

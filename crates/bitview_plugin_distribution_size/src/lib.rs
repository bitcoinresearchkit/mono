mod dependencies;
mod live;
pub use dependencies::Dependencies;
mod has;
pub use has::HasDistributionSize;
mod addr;
mod addr_groups;
mod block;
mod compute;
mod cumulative;
mod cumulative_value;
mod groups;
mod metrics;
mod sources;
mod state;
mod values;
mod vecs;
use bitview_plugin::{PluginId, PluginStorage};
use brk_oracle::VERSION as ORACLE_VERSION;
use brk_types::Version;
pub use vecs::Vecs;
const STORAGE: PluginStorage = PluginStorage::new(
    PluginId::new("distribution_size"),
    Version::new(41 + ORACLE_VERSION),
);
pub const ID: PluginId = STORAGE.id();
const SAVED_CHECKPOINTS: u16 = 10;

#[cfg(test)]
mod test_cache;

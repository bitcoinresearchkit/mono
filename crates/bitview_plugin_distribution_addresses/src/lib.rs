#![doc = include_str!("../README.md")]
mod addr;
mod block;
mod compute;
mod dependencies;
mod has;
mod live;
mod metrics;
mod state;
mod vecs;
use bitview_cohort::AmountRangeId;
use bitview_plugin::{PluginId, PluginStorage};
use brk_oracle::VERSION as ORACLE_VERSION;
use brk_types::Version;
pub use dependencies::Dependencies;
pub use has::HasDistributionAddresses;
pub use vecs::Vecs;
const STORAGE: PluginStorage = PluginStorage::new(
    PluginId::new("distribution_addresses"),
    Version::new(41 + ORACLE_VERSION),
);
pub const ID: PluginId = STORAGE.id();
const SAVED_CHECKPOINTS: u16 = 10;
const CAP_COUNT: usize = AmountRangeId::ALL.len();
#[cfg(test)]
mod test_cache;

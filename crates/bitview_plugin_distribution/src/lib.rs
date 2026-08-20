#![allow(clippy::type_complexity)]

mod addr;
mod all_chain_sources;
mod block;
mod compute;
mod dependencies;
mod has;
mod inner;
mod metrics;
mod state;
mod vecs;

use addr::{AddrsDataVecs, AnyAddrIndexesVecs};
pub use all_chain_sources::AllChainSources;
pub use dependencies::Dependencies;
pub use has::HasDistribution;
use metrics::CohortMetrics;
pub use state::UTXOStates;
pub use vecs::Vecs;

use bitview_plugin::{PluginId, PluginStorage};
use brk_oracle::VERSION as ORACLE_VERSION;
use brk_types::Version;

const STORAGE: PluginStorage = PluginStorage::new(
    PluginId::new("distribution"),
    Version::new(39 + ORACLE_VERSION),
);
pub const ID: PluginId = STORAGE.id();

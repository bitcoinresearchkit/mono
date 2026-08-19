#![allow(clippy::type_complexity)]

mod addr;
mod all_chain_sources;
mod block;
mod compute;
mod dependencies;
mod inner;
mod metrics;
mod state;
mod vecs;

use addr::{AddrsDataVecs, AnyAddrIndexesVecs};
pub use all_chain_sources::AllChainSources;
pub use dependencies::Dependencies;
use metrics::CohortMetrics;
pub use state::UTXOStates;
pub use vecs::Vecs;

pub const ID: bitview_plugin::PluginId = bitview_plugin::PluginId::new("distribution");
const DB_NAME: &str = ID.as_str();

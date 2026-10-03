mod block;
mod compute;
mod dependencies;
mod has;
mod metrics;
mod sources;
mod state;
mod type_sources;
mod vecs;

use bitview_cohort::{AmountRangeId, SpendableTypeId};
use bitview_plugin::{PluginId, PluginStorage};
use brk_types::Version;

pub use dependencies::Dependencies;
pub use has::HasDistributionUtxos;
pub use vecs::Vecs;

const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("distribution_utxos"), Version::new(47));
pub const ID: PluginId = STORAGE.id();
const SAVED_CHECKPOINTS: u16 = 10;
const CAP_COUNT: usize = AmountRangeId::ALL.len() + SpendableTypeId::ALL.len();

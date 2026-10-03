mod block;
mod checkpoint;
mod compute;
mod dependencies;
mod has;
mod import;
mod metrics;
mod sources;
mod state;
mod type_sources;

pub use dependencies::Dependencies;
pub use has::HasDistributionUtxos;

use bitview_cohort::{AmountRangeId, SpendableTypeId};
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_plugin_distribution_common::{RealizedCaps, replay::LiveState};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use metrics::CohortMetrics;
use state::UTXOStates;

const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("distribution_utxos"), Version::new(47));
pub const ID: PluginId = STORAGE.id();
const SAVED_CHECKPOINTS: u16 = 10;
const CAP_COUNT: usize = AmountRangeId::ALL.len() + SpendableTypeId::ALL.len();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState<UTXOStates>>>,
    caps: M::WriteOnly<RealizedCaps<CAP_COUNT>>,
    pub cohorts: CohortMetrics<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

impl Vecs {
    fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

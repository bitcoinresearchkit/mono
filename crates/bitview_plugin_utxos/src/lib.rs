mod block;
mod checkpoint;
mod compute;
mod dependencies;
mod import;
mod metrics;
mod state;

pub use dependencies::Dependencies;

use bitview_cohort::{AmountRangeId, SpendableTypeId};
use bitview_distribution::{RealizedCaps, replay::LiveState};
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use metrics::CohortMetrics;
use state::UTXOStates;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("utxos"), Version::new(48));
pub const ID: PluginId = STORAGE.id();
const SAVED_CHECKPOINTS: u16 = 10;
const CAP_COUNT: usize = AmountRangeId::ALL.len() + SpendableTypeId::ALL.len();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState<UTXOStates>>>,
    caps: M::WriteOnly<RealizedCaps<CAP_COUNT>>,
    #[traversable(flatten)]
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
        self.db.flush();
        Ok(())
    }
}

use brk_error::Result;
use brk_store::PendingIngest;
use brk_types::Height;
use fjall::Database;
use rayon::prelude::*;

use super::{PersistedStoresCheckpoint, StoresCheckpoint};

#[must_use = "persist this deferred commit before publishing its checkpoint"]
pub struct DeferredStoresCommit {
    checkpoint: StoresCheckpoint,
    completed_height: Height,
    database: Database,
    ingests: Vec<PendingIngest>,
}

impl DeferredStoresCommit {
    pub fn new(
        database: Database,
        ingests: Vec<PendingIngest>,
        checkpoint: StoresCheckpoint,
        completed_height: Height,
    ) -> Self {
        Self {
            checkpoint,
            completed_height,
            database,
            ingests,
        }
    }

    pub fn persist(self) -> Result<PersistedStoresCheckpoint> {
        let Self {
            checkpoint,
            completed_height,
            database,
            ingests,
        } = self;
        let persisted = checkpoint
            .begin(completed_height)?
            .persist(|| ingests.into_par_iter().try_for_each(PendingIngest::run));
        drop(database);
        persisted
    }
}

use brk_error::Result;
use brk_store::PendingIngest;
use fjall::Database;
use rayon::prelude::*;

use super::{PendingStoresCheckpoint, PersistedStoresCheckpoint};

#[must_use = "persist this deferred commit before publishing its checkpoint"]
pub(crate) struct DeferredStoresCommit {
    checkpoint: PendingStoresCheckpoint,
    database: Database,
    ingests: Vec<PendingIngest>,
}

impl DeferredStoresCommit {
    pub(crate) fn new(
        database: Database,
        ingests: Vec<PendingIngest>,
        checkpoint: PendingStoresCheckpoint,
    ) -> Self {
        Self {
            checkpoint,
            database,
            ingests,
        }
    }

    pub(crate) fn persist(self) -> Result<PersistedStoresCheckpoint> {
        let Self {
            checkpoint,
            database,
            ingests,
        } = self;
        let persisted =
            checkpoint.persist(|| ingests.into_par_iter().try_for_each(PendingIngest::run));
        drop(database);
        persisted
    }
}

use std::{panic::resume_unwind, thread};

use brk_error::Result;
use brk_store::PendingIngest;
use brk_types::Height;
use fjall::Database;

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
        // Each ingestion blocks on file writes: OS threads keep them off the rayon pool, which
        // the dependents computing beside them join on.
        let persisted = checkpoint.begin(completed_height)?.persist(|| {
            thread::scope(|scope| {
                let ingests = ingests
                    .into_iter()
                    .map(|ingest| scope.spawn(move || ingest.run()))
                    .collect::<Vec<_>>();
                ingests.into_iter().try_for_each(|ingest| {
                    ingest.join().unwrap_or_else(|panic| resume_unwind(panic))
                })
            })
        });
        drop(database);
        persisted
    }
}

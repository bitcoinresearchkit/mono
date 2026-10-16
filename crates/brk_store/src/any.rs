use brk_error::Result;

use crate::PendingIngest;

/// A store as its owner's commits see it, so one list of stores serves every commit step.
pub trait AnyStore: Send + Sync {
    /// Whether the keyspace holds no ingested records (pending writes aside).
    fn is_empty(&self) -> Result<bool>;

    /// Takes the buffered puts/dels into an ingestion to run anywhere, leaving empty buffers for
    /// the next batch; `None` when nothing is pending. A successful ingestion persists and
    /// publishes the records: it writes immutable tables and its manifest.
    fn take_pending_ingest(&mut self) -> Option<PendingIngest>;
}

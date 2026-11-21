use brk_error::Result;
use brk_exit::Exit;

/// What a commit's ingestions wait on before publishing their tables: they write beside it, and
/// publish once it returns `Ok`.
pub type Ready<'a> = &'a (dyn Fn() -> Result<()> + Sync);

/// One keyspace's ingestion, given what it waits on and whether to write only changes.
type Ingest = dyn FnOnce(Ready<'_>, bool) -> Result<()> + Send;

pub struct PendingIngest(Box<Ingest>);

impl PendingIngest {
    pub(crate) fn new(ingest: impl FnOnce(Ready<'_>, bool) -> Result<()> + Send + 'static) -> Self {
        Self(Box::new(ingest))
    }

    /// The stores' only write: one keyspace's pending records become a table, published once
    /// `ready` returns `Ok`.
    pub fn run(self, ready: Ready<'_>) -> Result<()> {
        debug_assert!(Exit::is_locked(), "store ingestion outside the exit lock");
        (self.0)(ready, false)
    }

    /// [`Self::run`] with only the records that change what the keyspace holds: a removal of an
    /// absent key and a put of the value already there are dropped, so running the same changes
    /// again writes nothing. It reads every key first.
    pub fn run_changes(self) -> Result<()> {
        debug_assert!(Exit::is_locked(), "store ingestion outside the exit lock");
        (self.0)(&|| Ok(()), true)
    }
}

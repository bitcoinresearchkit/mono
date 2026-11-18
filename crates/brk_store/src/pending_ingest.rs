use brk_error::Result;
use brk_exit::Exit;

pub struct PendingIngest(Box<dyn FnOnce() -> Result<()> + Send>);

impl PendingIngest {
    pub(crate) fn new(ingest: impl FnOnce() -> Result<()> + Send + 'static) -> Self {
        Self(Box::new(ingest))
    }

    /// The stores' only write: one keyspace's pending records become a table.
    pub fn run(self) -> Result<()> {
        debug_assert!(Exit::is_locked(), "store ingestion outside the exit lock");
        (self.0)()
    }
}

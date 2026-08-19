use brk_error::Result;

pub struct PendingIngest(Box<dyn FnOnce() -> Result<()> + Send>);

impl PendingIngest {
    pub fn new(ingest: impl FnOnce() -> Result<()> + Send + 'static) -> Self {
        Self(Box::new(ingest))
    }

    pub fn run(self) -> Result<()> {
        (self.0)()
    }
}

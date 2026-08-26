use brk_error::Result;

pub trait AnyStore: Send + Sync {
    fn ingest_pending(&mut self) -> Result<()>;
}

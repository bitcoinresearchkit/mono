pub trait AnyStore: Send + Sync {
    fn ingest_pending(&mut self) -> brk_error::Result<()>;
}

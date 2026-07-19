use brk_error::{Error, Result};
use brk_types::{Height, Version};
use fjall::Keyspace;

pub trait AnyStore: Send + Sync {
    fn name(&self) -> &'static str;
    fn height(&self) -> Option<Height>;
    fn has(&self, height: Height) -> bool;
    fn needs(&self, height: Height) -> bool;
    fn version(&self) -> Version;
    fn export_meta(&mut self, height: Height) -> Result<()>;
    #[doc(hidden)]
    fn export_meta_sync(&mut self, height: Height) -> Result<()> {
        self.export_meta(height)
    }
    fn export_meta_if_needed(&mut self, height: Height) -> Result<()>;
    #[doc(hidden)]
    fn ingest_pending(&mut self) -> Result<()> {
        Err(Error::Internal("pending ingestion is not implemented"))
    }
    fn keyspace(&self) -> &Keyspace;
    fn commit(&mut self, height: Height) -> Result<()>;
}

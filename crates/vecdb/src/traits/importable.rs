use rawdb::Database;

use crate::{ImportOptions, Result, Version};

pub trait ImportableVec: Sized {
    /// Import from database, creating if needed.
    ///
    /// # Warning
    ///
    /// Deletes the existing data, sidecar regions and rollback history included, when its
    /// version, format or value size differs (compressed vectors also reset a corrupt
    /// layout); other corruption is an error.
    fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Self::import_with((db, name, version).into())
    }

    /// Import with custom options; see [`Self::import`].
    fn import_with(options: ImportOptions) -> Result<Self>;
}

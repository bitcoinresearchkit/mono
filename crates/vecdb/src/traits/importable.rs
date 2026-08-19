use rawdb::Database;

use crate::{ImportOptions, Version};

/// Trait for types that can be imported from a database.
///
/// This provides a uniform interface for constructing stored vectors,
/// enabling generic wrappers like `EagerVec` to work with any storage format.
pub trait ImportableVec: Sized {
    /// Import from database, creating if needed.
    fn import(db: &Database, name: &str, version: Version) -> crate::Result<Self>;

    /// Import with custom options.
    fn import_with(options: ImportOptions) -> crate::Result<Self>;

    /// Import from database, resetting on version/format mismatch.
    fn forced_import(db: &Database, name: &str, version: Version) -> crate::Result<Self>;

    /// Forced import with custom options.
    fn forced_import_with(options: ImportOptions) -> crate::Result<Self>;
}

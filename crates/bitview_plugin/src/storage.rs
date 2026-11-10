use std::{fs, io::ErrorKind, path::PathBuf};

use brk_error::{Error, Result};
use vecdb::{Database, PAGE_SIZE, Version};

use crate::{ImportContext, PLUGIN_DATA_DIR, PluginId};

/// Stable identity and root storage schema for one plugin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginStorage {
    id: PluginId,
    schema_version: Version,
}

impl PluginStorage {
    pub const fn new(id: PluginId, schema_version: Version) -> Self {
        Self { id, schema_version }
    }

    pub const fn id(self) -> PluginId {
        self.id
    }

    pub const fn schema_version(self) -> Version {
        self.schema_version
    }

    pub fn plugins_path(context: ImportContext<'_>) -> PathBuf {
        context.data_path().join(PLUGIN_DATA_DIR)
    }

    pub fn path(self, context: ImportContext<'_>) -> PathBuf {
        Self::plugins_path(context).join(self.id.as_str())
    }

    /// Rebuilds the whole owner once after a classified data failure. The failed import must
    /// own all of its database handles: they are dropped before this removes the directory.
    /// Locks and other operational errors propagate without deletion. Bootstrap holds the
    /// shutdown lock throughout; `after_reset` restores any source metadata before retrying.
    pub fn import_with_recovery<T>(
        self,
        context: ImportContext<'_>,
        import: impl Fn() -> Result<T>,
        after_reset: impl Fn(&Error) -> Result<()>,
    ) -> Result<T> {
        match import() {
            Err(error) if !error.is_lock_error() && error.is_data_error() => {
                match fs::remove_dir_all(self.path(context)) {
                    Ok(()) => {}
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
                after_reset(&error)?;
                import()
            }
            result => result,
        }
    }

    pub fn open_database(
        self,
        context: ImportContext<'_>,
        page_multiplier: usize,
    ) -> Result<Database> {
        let db = Database::open(&self.path(context))?;
        db.set_min_len(PAGE_SIZE * page_multiplier)?;
        Ok(db)
    }

    pub fn finalize_database(self, db: &Database) -> Result<()> {
        db.retain_accessed_regions()?;
        db.compact()?;
        Ok(())
    }
}

use std::path::PathBuf;

use brk_error::Result;
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

    /// Builds plugin data with an owned database handle and its root schema version.
    /// Retains accessed regions and compacts only after a successful build.
    pub fn import<T>(
        self,
        context: ImportContext<'_>,
        page_multiplier: usize,
        build: impl FnOnce(Database, Version) -> Result<T>,
    ) -> Result<T> {
        let db = self.open_database(context, page_multiplier)?;
        let data = build(db.clone(), self.schema_version())?;
        self.finalize_database(&db)?;
        Ok(data)
    }

    fn open_database(
        self,
        context: ImportContext<'_>,
        page_multiplier: usize,
    ) -> Result<Database> {
        let db = Database::open(&self.path(context))?;
        db.set_min_len(PAGE_SIZE * page_multiplier)?;
        Ok(db)
    }

    fn finalize_database(self, db: &Database) -> Result<()> {
        db.retain_accessed_regions()?;
        db.compact()?;
        Ok(())
    }
}

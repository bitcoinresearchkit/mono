use std::path::PathBuf;

use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, PAGE_SIZE};

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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use brk_types::Version;

    use super::*;

    #[test]
    fn identity_version_and_paths_share_one_descriptor() {
        let context = ImportContext::new(Path::new("data"));
        let storage = PluginStorage::new(PluginId::new("example"), Version::new(3));

        assert_eq!(storage.id(), PluginId::new("example"));
        assert_eq!(storage.schema_version(), Version::new(3));
        assert_eq!(
            PluginStorage::plugins_path(context),
            context.data_path().join("plugins")
        );
        assert_eq!(
            storage.path(context),
            context.data_path().join("plugins/example")
        );
    }
}

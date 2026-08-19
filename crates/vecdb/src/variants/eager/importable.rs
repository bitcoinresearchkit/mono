use rawdb::Database;

use crate::{ImportOptions, ImportableVec, Version};

use super::EagerVec;

impl<V: ImportableVec> ImportableVec for EagerVec<V> {
    fn import(db: &Database, name: &str, version: Version) -> crate::Result<Self> {
        Ok(Self(V::import(db, name, version)?))
    }

    fn import_with(options: ImportOptions) -> crate::Result<Self> {
        Ok(Self(V::import_with(options)?))
    }

    fn forced_import(db: &Database, name: &str, version: Version) -> crate::Result<Self> {
        Ok(Self(V::forced_import(db, name, version)?))
    }

    fn forced_import_with(options: ImportOptions) -> crate::Result<Self> {
        Ok(Self(V::forced_import_with(options)?))
    }
}

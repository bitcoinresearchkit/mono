use brk_error::Result;

use std::path::Path;

use brk_types::Version;

use bitview_compute::{
    CachedWindowStartVec, LazyPerSecondWindows, Windows,
    db_utils::{finalize_db, open_db},
};

use super::Vecs;

impl Vecs {
    pub fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = open_db(parent_path, super::DB_NAME, 20_000_000)?;
        let version = parent_version;

        let spent = super::spent::forced_import(&db, version)?;
        let count = super::count::forced_import(&db, version, indexes, cached_starts)?;
        let per_sec =
            LazyPerSecondWindows::new("outputs_per_sec", version, &count.total.rolling.sum);
        let unspent = super::unspent::forced_import(&db, version, indexes)?;
        let by_type = super::by_type::forced_import(&db, version, indexes, cached_starts)?;
        let value = super::value::forced_import(&db, version, indexes)?;

        let this = Self {
            plugin_gate: Default::default(),
            db,
            spent,
            count,
            per_sec,
            unspent,
            by_type,
            value,
        };
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}

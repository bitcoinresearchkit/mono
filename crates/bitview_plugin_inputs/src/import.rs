use brk_error::Result;

use std::path::Path;

use bitview_compute::{
    CachedWindowStartVec, LazyPerSecondWindows, Windows,
    db_utils::{finalize_db, open_db},
};
use brk_types::Version;
use vecdb::{ImportableVec, PcoVec};

use super::{ByTypeVecs, CountVecs, Vecs};

impl Vecs {
    pub fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = open_db(parent_path, super::DB_NAME, 20_000_000)?;
        let version = parent_version;

        let value = PcoVec::forced_import(&db, "value", version)?;
        let count = CountVecs::forced_import(&db, version, indexes, cached_starts)?;
        let per_sec = LazyPerSecondWindows::new("inputs_per_sec", version, &count.rolling.sum);
        let by_type = ByTypeVecs::forced_import(&db, version, indexes, cached_starts)?;

        let this = Self {
            plugin_gate: Default::default(),
            db,
            value,
            count,
            per_sec,
            by_type,
        };
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}

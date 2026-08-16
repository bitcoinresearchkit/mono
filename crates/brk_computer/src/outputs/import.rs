use std::path::Path;

use brk_error::Result;
use brk_types::Version;

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, LazyPerSecondWindows, Windows,
        db_utils::{finalize_db, open_db},
    },
};

use super::{ByTypeVecs, CountVecs, SpentVecs, UnspentVecs, ValueVecs, Vecs};

impl Vecs {
    pub(crate) fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = open_db(parent_path, super::DB_NAME, 20_000_000)?;
        let version = parent_version;

        let spent = SpentVecs::forced_import(&db, version)?;
        let count = CountVecs::forced_import(&db, version, indexes, cached_starts)?;
        let per_sec =
            LazyPerSecondWindows::new("outputs_per_sec", version, &count.total.rolling.sum);
        let unspent = UnspentVecs::forced_import(&db, version, indexes)?;
        let by_type = ByTypeVecs::forced_import(&db, version, indexes, cached_starts)?;
        let value = ValueVecs::forced_import(&db, version, indexes)?;

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

use brk_error::Result;

use std::path::Path;

use bitview_compute::{
    CachedWindowStartVec, Windows,
    db_utils::{finalize_db, open_db},
};
use bitview_plugin_indexer::Indexer;
use brk_types::Version;

use super::Vecs;

impl Vecs {
    pub fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexer: &Indexer,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = open_db(parent_path, super::ID.as_str(), 10_000_000)?;
        let version = parent_version;

        let count = super::count::forced_import(&db, version, indexes, cached_starts)?;
        let features = super::features::forced_import(&db, version, indexes, cached_starts)?;
        let size = super::size::forced_import(&db, version, indexer, indexes)?;
        let fees = super::fees::forced_import(&db, version, indexes, cached_starts)?;
        let patterns = super::patterns::forced_import(&db, version, indexes, cached_starts)?;
        let policy = super::policy::forced_import(&db, version, indexes, cached_starts)?;
        let sigops = super::sigops::forced_import(&db, version, indexes, cached_starts)?;
        let versions = super::versions::forced_import(&db, version, indexes, cached_starts)?;
        let volume = super::volume::forced_import(
            &db,
            version,
            indexes,
            cached_starts,
            &count.total.rolling.sum,
        )?;

        let this = Self {
            plugin_gate: Default::default(),
            db,
            count,
            features,
            size,
            fees,
            patterns,
            policy,
            sigops,
            versions,
            volume,
        };
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}

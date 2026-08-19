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
        let db = open_db(parent_path, super::ID.as_str(), 1_000_000)?;
        let version = parent_version;

        let rewards = super::rewards::forced_import(&db, version, indexer, indexes, cached_starts)?;
        let hashrate = super::hashrate::forced_import(&db, version, indexes)?;

        let this = Self {
            plugin_gate: Default::default(),
            db,
            rewards,
            hashrate,
        };
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}

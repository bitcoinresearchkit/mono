use std::path::Path;

use brk_error::Result;
use brk_indexer::Indexer;
use brk_types::Version;

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, Windows,
        db_utils::{finalize_db, open_db},
    },
};

use super::{HashrateVecs, RewardsVecs, Vecs};

impl Vecs {
    pub(crate) fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = open_db(parent_path, super::DB_NAME, 1_000_000)?;
        let version = parent_version;

        let rewards = RewardsVecs::forced_import(&db, version, indexer, indexes, cached_starts)?;
        let hashrate = HashrateVecs::forced_import(&db, version, indexes)?;

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

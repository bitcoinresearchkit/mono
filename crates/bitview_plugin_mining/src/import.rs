use brk_error::Result;

use bitview_compute::{CachedWindowStartVec, Windows};
use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;

use super::{STORAGE, Vecs};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        indexer: &Indexer,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();

        let rewards =
            super::rewards::forced_import(&db, version, indexer, mappings, cached_starts)?;
        let hashrate = super::hashrate::forced_import(&db, version, mappings)?;

        let this = Self {
            plugin_gate: Default::default(),
            db,
            rewards,
            hashrate,
        };
        STORAGE.finalize_database(&this.db, &this)?;
        Ok(this)
    }
}

use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;

use super::{STORAGE, Vecs, hashrate, rewards};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();

        let rewards = rewards::Vecs::import(&db, version, mappings, window_starts)?;
        let hashrate = hashrate::Vecs::import(&db, version, mappings)?;

        let this = Self {
            db,
            rewards,
            hashrate,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

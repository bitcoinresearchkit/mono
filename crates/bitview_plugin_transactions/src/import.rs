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
        let db = STORAGE.open_database(context, 10_000_000)?;
        let version = STORAGE.schema_version();

        let count = super::count::forced_import(&db, version, mappings, cached_starts)?;
        let features = super::features::forced_import(&db, version, mappings, cached_starts)?;
        let size = super::size::forced_import(&db, version, indexer, mappings)?;
        let fees = super::fees::forced_import(&db, version, mappings, cached_starts)?;
        let patterns = super::patterns::forced_import(&db, version, mappings, cached_starts)?;
        let policy = super::policy::forced_import(&db, version, mappings, cached_starts)?;
        let sigops = super::sigops::forced_import(&db, version, mappings, cached_starts)?;
        let versions = super::versions::forced_import(&db, version, mappings, cached_starts)?;
        let volume = super::volume::forced_import(
            &db,
            version,
            mappings,
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
        STORAGE.finalize_database(&this.db, &this)?;
        Ok(this)
    }
}

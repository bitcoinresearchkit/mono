use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;

use super::{
    STORAGE, Vecs, count, features, fees, inscription, patterns, policy, sigops, size, versions,
    volume,
};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 10_000_000)?;
        let version = STORAGE.schema_version();

        let count = count::forced_import(&db, version, mappings, window_starts)?;
        let features = features::forced_import(&db, version, mappings, window_starts)?;
        let size = size::forced_import(&db, version, indexer, mappings)?;
        let fees = fees::forced_import(&db, version, mappings, window_starts)?;
        let inscription = inscription::forced_import(&db, version, mappings, window_starts)?;
        let patterns = patterns::forced_import(&db, version, mappings, window_starts)?;
        let policy = policy::forced_import(&db, version, mappings, window_starts)?;
        let sigops = sigops::forced_import(&db, version, mappings, window_starts)?;
        let versions = versions::forced_import(&db, version, mappings, window_starts)?;
        let volume = volume::forced_import(
            &db,
            version,
            mappings,
            window_starts,
            &count.total.rolling.sum,
        )?;

        let this = Self {
            db,
            count,
            features,
            size,
            fees,
            inscription,
            patterns,
            policy,
            sigops,
            versions,
            volume,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

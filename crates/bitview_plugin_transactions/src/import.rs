use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyPerSecondWindows, LazyWindowStartVec, ValuePerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::Version;

use super::{
    STORAGE, Vecs, count, features, fees, inscription, patterns, policy, sigops, size, versions,
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

        let count = count::Vecs::import(&db, version, mappings, window_starts)?;
        let features = features::Vecs::import(&db, version, indexer, mappings, window_starts)?;
        let size = size::Vecs::import(&db, version, indexer, mappings)?;
        let fees = fees::Vecs::import(&db, version, mappings, window_starts)?;
        let inscription =
            inscription::Vecs::import(&db, version, indexer, mappings, window_starts)?;
        let patterns = patterns::Vecs::import(&db, version, mappings, window_starts)?;
        let policy = policy::Vecs::import(&db, version, mappings, window_starts)?;
        let sigops = sigops::Vecs::import(&db, version, mappings, window_starts)?;
        let versions = versions::Vecs::import(&db, version, mappings, window_starts)?;
        let volume = ValuePerBlockCumulativeRolling::import(
            &db,
            "tx_volume",
            version,
            mappings,
            window_starts,
        )?;
        let per_second =
            LazyPerSecondWindows::new("tx_per_second", version + Version::TWO, &count.rolling.sum);

        let this = Self {
            db,
            count,
            features,
            size,
            fees,
            inscription,
            patterns,
            policy,
            sigop_cost: sigops,
            versions,
            volume,
            per_second,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

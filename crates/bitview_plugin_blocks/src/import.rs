use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;

use super::{
    DifficultyVecs, HalvingVecs, IntervalVecs, LookbackVecs, STORAGE, SizeVecs, Vecs, WeightVecs,
    count,
};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        indexer: &Indexer,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();

        let lookback = LookbackVecs::new(
            version,
            mappings.timestamp.monotonic.read_only_boxed_clone(),
        );
        let window_starts = lookback.window_starts();
        let count = count::block_count(version, indexer, mappings, &window_starts);
        let interval = IntervalVecs::import(&db, version, mappings, &window_starts)?;
        let weight = WeightVecs::import(&db, version, indexer, mappings, &window_starts)?;
        let size = SizeVecs::import(&db, version, indexer, mappings, &window_starts, &weight)?;
        let difficulty = DifficultyVecs::new(version, indexer, mappings);
        let halving = HalvingVecs::new(version, mappings);

        let this = Self {
            db,
            count,
            lookback,
            interval,
            size,
            weight,
            difficulty,
            halving,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
use vecdb::ReadableCloneableVec;

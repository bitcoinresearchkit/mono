use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;

use super::{
    CountVecs, DifficultyVecs, HalvingVecs, IntervalVecs, LookbackVecs, STORAGE, UtxosVecs, Vecs,
    WeightVecs,
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
        let count = CountVecs::new(version, indexer, mappings, &window_starts);
        let interval = IntervalVecs::forced_import(&db, version, mappings, &window_starts)?;
        let size = UtxosVecs::forced_import(&db, version, indexer, mappings, &window_starts)?;
        let weight = WeightVecs::new(version, indexer, mappings, &window_starts, &size);
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

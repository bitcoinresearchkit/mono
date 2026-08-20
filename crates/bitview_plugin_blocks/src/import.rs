use brk_error::Result;

use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;

use super::{
    CountVecs, DifficultyVecs, HalvingVecs, IntervalVecs, LookbackVecs, STORAGE, SizeVecs, Vecs,
    WeightVecs,
};
use super::{
    count::Import as _, difficulty::Import as _, halving::Import as _, interval::Import as _,
    lookback::Internal as _, size::Import as _, weight::Import as _,
};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        indexer: &Indexer,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();

        let lookback = LookbackVecs::new(
            version,
            mappings.timestamp.monotonic.read_only_cached_boxed_clone(),
        );
        let cached_starts = lookback.cached_window_starts();
        let count = CountVecs::new(version, indexer, mappings, &cached_starts);
        let interval = IntervalVecs::forced_import(&db, version, mappings, &cached_starts)?;
        let size = SizeVecs::forced_import(&db, version, indexer, mappings, &cached_starts)?;
        let weight = WeightVecs::new(version, indexer, mappings, &cached_starts, &size);
        let difficulty = DifficultyVecs::new(version, indexer, mappings);
        let halving = HalvingVecs::new(version, mappings);

        let this = Self {
            plugin_gate: Default::default(),
            db,
            count,
            lookback,
            interval,
            size,
            weight,
            difficulty,
            halving,
        };
        STORAGE.finalize_database(&this.db, &this)?;
        Ok(this)
    }
}

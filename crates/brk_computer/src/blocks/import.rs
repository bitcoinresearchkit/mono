use std::path::Path;

use brk_error::Result;
use brk_indexer::Indexer;
use brk_types::Version;

use crate::{
    indexes,
    internal::db_utils::{finalize_db, open_db},
};

use super::{
    CountVecs, DifficultyVecs, HalvingVecs, IntervalVecs, LookbackVecs, SizeVecs, Vecs, WeightVecs,
};

impl Vecs {
    pub(crate) fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let db = open_db(parent_path, super::DB_NAME, 1_000_000)?;
        let version = parent_version;

        let lookback = LookbackVecs::new(
            version,
            indexes.timestamp.monotonic.read_only_cached_boxed_clone(),
        );
        let cached_starts = lookback.cached_window_starts();
        let count = CountVecs::new(version, indexer, indexes, &cached_starts);
        let interval = IntervalVecs::forced_import(&db, version, indexes, &cached_starts)?;
        let size = SizeVecs::forced_import(&db, version, indexer, indexes, &cached_starts)?;
        let weight = WeightVecs::new(version, indexer, indexes, &cached_starts, &size);
        let difficulty = DifficultyVecs::new(version, indexer, indexes);
        let halving = HalvingVecs::new(version, indexes);

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
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}

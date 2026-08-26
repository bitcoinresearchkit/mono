use brk_error::Result;

use bitview_compute::{CachedWindowStartVec, LazyPerSecondWindows, Windows};
use bitview_plugin::ImportContext;
use vecdb::{ImportableVec, PcoVec};

use super::{ByTypeVecs, CountVecs, STORAGE, Vecs};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();

        let value = PcoVec::forced_import(&db, "value", version)?;
        let count = CountVecs::forced_import(&db, version, mappings, cached_starts)?;
        let per_sec = LazyPerSecondWindows::new("inputs_per_sec", version, &count.rolling.sum);
        let by_type = ByTypeVecs::forced_import(&db, version, mappings, cached_starts)?;

        let this = Self {
            plugin_gate: Default::default(),
            db,
            value,
            count,
            per_sec,
            by_type,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

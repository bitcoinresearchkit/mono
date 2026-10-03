use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyPerSecondWindows, LazyWindowStartVec};
use brk_error::Result;
use vecdb::{ImportableVec, PcoVec};

use super::{ByTypeVecs, CountVecs, STORAGE, Vecs};
use crate::OriginSpends;

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();

        let value = PcoVec::forced_import(&db, "value", version)?;
        let count = CountVecs::forced_import(&db, version, mappings, window_starts)?;
        let per_sec = LazyPerSecondWindows::new("inputs_per_sec", version, &count.rolling.sum);
        let by_type = ByTypeVecs::forced_import(&db, version, mappings, window_starts)?;

        let origins = OriginSpends::open(db.path())?;
        let this = Self {
            origins,
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

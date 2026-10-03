use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyPerSecondWindows, LazyWindowStartVec};
use brk_error::Result;
use statedb::Creations;

use super::{STORAGE, Vecs, by_type, count, spent, value};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();

        let spent = spent::Vecs::import(&db, version)?;
        let count = count::Vecs::import(&db, version, mappings, window_starts)?;
        let per_sec =
            LazyPerSecondWindows::new("outputs_per_sec", version, &count.total.rolling.sum);
        let by_type = by_type::Vecs::import(&db, version, mappings, window_starts)?;
        let value = value::Vecs::import(&db, version, mappings)?;

        let creations = Creations::open(db.path())?;
        let this = Self {
            creations,
            db,
            spent,
            count,
            per_sec,
            by_type,
            value,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

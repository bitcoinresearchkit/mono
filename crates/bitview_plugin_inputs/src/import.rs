use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyPerSecondWindows, LazyWindowStartVec};
use brk_error::Result;
use vecdb::{ImportableVec, PcoVec};

use super::{CountVecs, STORAGE, TypesVecs, Vecs};
use crate::OriginSpends;

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();

        let value = PcoVec::import(&db, "input_value", version)?;
        let count = CountVecs::import(&db, version, mappings, window_starts)?;
        let per_second =
            LazyPerSecondWindows::new("inputs_per_second", version, &count.rolling.sum);
        let types = TypesVecs::import(
            &db,
            version,
            mappings,
            window_starts,
            &count.cumulative.height,
        )?;

        let origins = OriginSpends::open(db.path())?;
        let this = Self {
            origins,
            db,
            value,
            count,
            per_second,
            types,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

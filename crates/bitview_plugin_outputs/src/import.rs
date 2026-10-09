use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyPerSecondWindows, LazyWindowStartVec};
use brk_error::Result;
use brk_types::Version;
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
        let per_second =
            LazyPerSecondWindows::new("outputs_per_second", version, &count.rolling.sum);
        let types = by_type::Vecs::import(&db, version, mappings, window_starts)?;
        let spendable_count = by_type::SpendableOutputCount::new(
            version + Version::TWO,
            &types.types.unspendable.op_return.count.cumulative.height,
            mappings,
            window_starts,
        );
        let value = value::Vecs::import(&db, version, mappings)?;

        let creations = Creations::open(db.path())?;
        let this = Self {
            creations,
            db,
            spent,
            count,
            per_second,
            spendable_count,
            types,
            value,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

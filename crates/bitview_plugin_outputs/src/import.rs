use brk_error::Result;

use bitview_compute::{CachedWindowStartVec, LazyPerSecondWindows, Windows};
use bitview_plugin::ImportContext;

use super::{STORAGE, Vecs};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();

        let spent = super::spent::forced_import(&db, version)?;
        let count = super::count::forced_import(&db, version, mappings, cached_starts)?;
        let per_sec =
            LazyPerSecondWindows::new("outputs_per_sec", version, &count.total.rolling.sum);
        let unspent = super::unspent::forced_import(&db, version, mappings)?;
        let by_type = super::by_type::forced_import(&db, version, mappings, cached_starts)?;
        let value = super::value::forced_import(&db, version, mappings)?;

        let this = Self {
            plugin_gate: Default::default(),
            db,
            spent,
            count,
            per_sec,
            unspent,
            by_type,
            value,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

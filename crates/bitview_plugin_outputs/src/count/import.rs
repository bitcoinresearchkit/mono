use brk_error::Result;

use brk_types::Version;
use vecdb::Database;

use super::Vecs;
use bitview_compute::{CachedWindowStartVec, PerBlockAggregated, Windows};

pub fn forced_import(
    db: &Database,
    version: Version,
    indexes: &bitview_plugin_indexes::Vecs,
    cached_starts: &Windows<&CachedWindowStartVec>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, indexes, cached_starts)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            total: PerBlockAggregated::forced_import(
                db,
                "output_count",
                version,
                indexes.output_count_source(),
                indexes,
                cached_starts,
            )?,
        })
    }
}

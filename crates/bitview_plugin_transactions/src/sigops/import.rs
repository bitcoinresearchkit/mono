use brk_error::Result;

use bitview_compute::{CachedWindowStartVec, PerBlockCumulativeRolling, Windows};
use brk_types::Version;
use vecdb::Database;

use super::Vecs;

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
            total: PerBlockCumulativeRolling::forced_import(
                db,
                "total_sigop_cost",
                version,
                indexes,
                cached_starts,
            )?,
        })
    }
}

use brk_error::Result;

use brk_types::Version;
use vecdb::Database;

use super::Vecs;
use bitview_compute::{CachedWindowStartVec, PerBlockCumulativeRolling, Windows};

pub fn forced_import(
    db: &Database,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
    cached_starts: &Windows<&CachedWindowStartVec>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, mappings, cached_starts)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            destroyed: PerBlockCumulativeRolling::forced_import(
                db,
                "cointime_value_destroyed",
                version,
                mappings,
                cached_starts,
            )?,
            created: PerBlockCumulativeRolling::forced_import(
                db,
                "cointime_value_created",
                version,
                mappings,
                cached_starts,
            )?,
            stored: PerBlockCumulativeRolling::forced_import(
                db,
                "cointime_value_stored",
                version,
                mappings,
                cached_starts,
            )?,
            vocdd: PerBlockCumulativeRolling::forced_import(
                db,
                "vocdd",
                version + Version::ONE,
                mappings,
                cached_starts,
            )?,
        })
    }
}

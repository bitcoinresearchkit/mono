use brk_error::Result;

use bitview_compute::{CachedWindowStartVec, PerBlockCumulativeRolling, Windows};
use brk_types::Version;
use vecdb::{Database, EagerVec, ImportableVec};

use super::{CountVecs, Vecs};

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
            count: CountVecs {
                nonstandard: PerBlockCumulativeRolling::forced_import(
                    db,
                    "nonstandard_count",
                    version,
                    indexes,
                    cached_starts,
                )?,
            },
            is_nonstandard: EagerVec::forced_import(db, "is_nonstandard", version)?,
        })
    }
}

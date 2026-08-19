use brk_error::Result;

use bitview_compute::{CachedWindowStartVec, PerBlockFullFromCumulative, Windows};
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
            total: PerBlockFullFromCumulative::forced_import(
                db,
                "tx_count",
                version,
                indexes.transaction_count_source(),
                indexes,
                cached_starts,
            )?,
        })
    }
}

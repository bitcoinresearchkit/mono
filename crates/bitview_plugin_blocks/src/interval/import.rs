use brk_error::Result;

use bitview_compute::{CachedWindowStartVec, PerBlockCumulativeAverage, Windows};
use brk_types::Version;
use vecdb::Database;

use super::Vecs;

pub trait Import: Sized {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self>;
}

impl Import for Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let interval = PerBlockCumulativeAverage::forced_import(
            db,
            "block_interval",
            version,
            mappings,
            cached_starts,
        )?;

        Ok(Self(interval))
    }
}

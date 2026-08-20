use brk_error::Result;

use bitview_compute::{CachedWindowStartVec, PerBlockAggregated, Windows};
use brk_types::Version;
use vecdb::Database;

use super::Vecs;

impl Vecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self(PerBlockAggregated::forced_import(
            db,
            "input_count",
            version,
            mappings.input_count_source(),
            mappings,
            cached_starts,
        )?))
    }
}

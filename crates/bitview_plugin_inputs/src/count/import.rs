use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockAggregated};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::Vecs;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self(PerBlockAggregated::import(
            db,
            "input_count",
            version,
            &mappings.input_count_source(),
            mappings,
            window_starts,
        )?))
    }
}

use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeAverage};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::Vecs;

impl Vecs {
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let interval = PerBlockCumulativeAverage::import(
            db,
            "block_interval",
            version,
            mappings,
            window_starts,
        )?;

        Ok(Self(interval))
    }
}

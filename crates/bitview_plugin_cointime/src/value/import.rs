use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
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
        Ok(Vecs {
            destroyed: PerBlockCumulativeRolling::import(
                db,
                "cointime_value_destroyed",
                version,
                mappings,
                window_starts,
            )?,
            created: PerBlockCumulativeRolling::import(
                db,
                "cointime_value_created",
                version,
                mappings,
                window_starts,
            )?,
            stored: PerBlockCumulativeRolling::import(
                db,
                "cointime_value_stored",
                version,
                mappings,
                window_starts,
            )?,
        })
    }
}

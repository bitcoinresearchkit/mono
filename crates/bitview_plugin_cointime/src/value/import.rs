use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::Vecs;

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Vecs {
            destroyed: PerBlockCumulativeRolling::forced_import(
                db,
                "cointime_value_destroyed",
                version,
                mappings,
                window_starts,
            )?,
            created: PerBlockCumulativeRolling::forced_import(
                db,
                "cointime_value_created",
                version,
                mappings,
                window_starts,
            )?,
            stored: PerBlockCumulativeRolling::forced_import(
                db,
                "cointime_value_stored",
                version,
                mappings,
                window_starts,
            )?,
            vocdd: PerBlockCumulativeRolling::forced_import(
                db,
                "vocdd",
                version + Version::ONE,
                mappings,
                window_starts,
            )?,
        })
    }
}

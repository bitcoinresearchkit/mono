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
        let import = |name| {
            PerBlockCumulativeRolling::import(
                db,
                name,
                version + Version::ONE,
                mappings,
                window_starts,
            )
        };
        Ok(Vecs {
            v1: import("tx_v1")?,
            v2: import("tx_v2")?,
            v3: import("tx_v3")?,
            other: import("tx_other_version")?,
        })
    }
}

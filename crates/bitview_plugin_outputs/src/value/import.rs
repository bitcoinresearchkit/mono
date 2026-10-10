use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, ValuePerBlockCumulativeRolling};
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
            op_return_value: ValuePerBlockCumulativeRolling::import(
                db,
                "op_return_value",
                version,
                mappings,
                window_starts,
            )?,
        })
    }
}

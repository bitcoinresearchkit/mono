use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{
    LazyPerSecondWindows, LazyRollingSumsFromHeight, LazyWindowStartVec,
    ValuePerBlockCumulativeRolling,
};
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
        tx_count_sums: &LazyRollingSumsFromHeight<Count>,
    ) -> Result<Self> {
        let v = version + Version::TWO;
        Ok(Vecs {
            value: ValuePerBlockCumulativeRolling::import(
                db,
                "tx_volume",
                version,
                mappings,
                window_starts,
            )?,
            tx_per_second: LazyPerSecondWindows::new("tx_per_second", v, tx_count_sums),
        })
    }
}

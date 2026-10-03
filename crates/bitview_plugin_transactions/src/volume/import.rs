use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{
    LazyPerSecondWindows, LazyRollingSumsFromHeight, LazyWindowStartVec,
    ValuePerBlockCumulativeRolling,
};
use brk_error::Result;
use brk_types::{StoredU64, Version};
use vecdb::Database;

use super::Vecs;

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        tx_count_sums: &LazyRollingSumsFromHeight<StoredU64>,
    ) -> Result<Self> {
        let v = version + Version::TWO;
        Ok(Vecs {
            transfer_volume: ValuePerBlockCumulativeRolling::forced_import(
                db,
                "transfer_volume_bis",
                version,
                mappings,
                window_starts,
            )?,
            tx_per_sec: LazyPerSecondWindows::new("tx_per_sec", v, tx_count_sums),
        })
    }
}

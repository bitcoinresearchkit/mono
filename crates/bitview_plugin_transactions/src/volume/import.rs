use brk_error::Result;

use bitview_compute::{
    CachedWindowStartVec, LazyPerSecondWindows, LazyRollingSumsFromHeight,
    ValuePerBlockCumulativeRolling, Windows,
};
use brk_types::{StoredU64, Version};
use vecdb::Database;

use super::Vecs;

pub fn forced_import(
    db: &Database,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
    cached_starts: &Windows<&CachedWindowStartVec>,
    tx_count_sums: &LazyRollingSumsFromHeight<StoredU64>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, mappings, cached_starts, tx_count_sums)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        tx_count_sums: &LazyRollingSumsFromHeight<StoredU64>,
    ) -> Result<Self> {
        let v = version + Version::TWO;
        Ok(Self {
            transfer_volume: ValuePerBlockCumulativeRolling::forced_import(
                db,
                "transfer_volume_bis",
                version,
                mappings,
                cached_starts,
            )?,
            tx_per_sec: LazyPerSecondWindows::new("tx_per_sec", v, tx_count_sums),
        })
    }
}

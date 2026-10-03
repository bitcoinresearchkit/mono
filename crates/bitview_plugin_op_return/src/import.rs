use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::StoredU64;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Height, Sats, Version};
use vecdb::ReadableCloneableVec;

use crate::{
    STORAGE, Vecs,
    breakdown::{KindBreakdownVecs, PolicyBreakdownVecs},
    total::Total,
};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        block_size: &impl ReadableCloneableVec<Height, StoredU64>,
        chain_fees: &impl ReadableCloneableVec<Height, Sats>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();
        let total = Total::import(
            &db,
            "op_return",
            version,
            mappings,
            window_starts,
            block_size,
            chain_fees,
        )?;
        let breakdown_version = version + Version::ONE;
        let total_data = total.data_bytes_source();
        let by_kind = KindBreakdownVecs::import(
            &db,
            "op_return",
            breakdown_version,
            mappings,
            window_starts,
            total_data,
            block_size,
            chain_fees,
        )?;
        let policy = PolicyBreakdownVecs::import(
            &db,
            "op_return_policy",
            breakdown_version,
            mappings,
            window_starts,
            total_data,
            block_size,
            chain_fees,
        )?;

        let this = Self {
            db,
            total,
            by_kind,
            policy,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

use bitview_compute::{CachedWindowStartVec, Windows};
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;
use brk_types::{Height, Sats, StoredU64, Version};
use vecdb::CachedBoxedVec;

use super::Vecs;
use crate::{STORAGE, breakdown::BreakdownVecs, total::Total};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        block_size: CachedBoxedVec<Height, StoredU64>,
        chain_fees: CachedBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();
        let total = Total::forced_import(
            &db,
            "op_return",
            version,
            mappings,
            cached_starts,
            &block_size,
            &chain_fees,
        )?;
        let columnar_version = version + Version::ONE;
        let total_data = total.cached_data_bytes();
        let by_kind = BreakdownVecs::forced_import(
            &db,
            "op_return_cumulative_by_kind",
            "op_return",
            columnar_version,
            mappings,
            cached_starts,
            &total_data,
            &block_size,
            &chain_fees,
        )?;
        let policy = BreakdownVecs::forced_import(
            &db,
            "op_return_cumulative_policy",
            "op_return_policy",
            columnar_version,
            mappings,
            cached_starts,
            &total_data,
            &block_size,
            &chain_fees,
        )?;

        let this = Self {
            plugin_gate: Default::default(),
            db,
            total,
            by_kind,
            policy,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

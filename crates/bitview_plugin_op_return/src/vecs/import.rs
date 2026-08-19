use brk_error::Result;

use std::path::Path;

use brk_types::{Height, Sats, StoredU64, Version};
use vecdb::CachedBoxedVec;

use super::Vecs;
use crate::{breakdown::BreakdownVecs, total::Total};
use bitview_compute::{
    CachedWindowStartVec, Windows,
    db_utils::{finalize_db, open_db},
};

impl Vecs {
    pub fn forced_import(
        parent_path: &Path,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        block_size: CachedBoxedVec<Height, StoredU64>,
        chain_fees: CachedBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let db = open_db(parent_path, crate::ID.as_str(), 1_000_000)?;
        let total = Total::forced_import(
            &db,
            "op_return",
            version,
            indexes,
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
            indexes,
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
            indexes,
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
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}

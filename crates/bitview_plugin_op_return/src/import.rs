use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Bytes;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Height, Sats};
use vecdb::ReadableCloneableVec;

use crate::{
    STORAGE, Vecs,
    breakdown::{KindBreakdownVecs, PolicyBreakdownVecs},
    total::Total,
};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        block_size: &impl ReadableCloneableVec<Height, Bytes>,
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
        let protocols = KindBreakdownVecs::import(
            &db,
            version,
            mappings,
            window_starts,
            block_size,
            chain_fees,
        )?;
        let policies = PolicyBreakdownVecs::import(
            &db,
            version,
            mappings,
            window_starts,
            block_size,
            chain_fees,
        )?;

        let flag = indexer.vecs().transactions.features.op_return.flag_view();

        let this = Self {
            db,
            flag,
            total,
            protocols,
            policies,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

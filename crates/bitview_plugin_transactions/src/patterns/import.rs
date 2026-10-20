use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, ReadableCloneableVec};

use super::Vecs;
use crate::flagged::Classified;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        all_txs: &impl ReadableCloneableVec<Height, Count>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let pattern = |flag, kind| {
            Classified::import(db, flag, kind, version, mappings, window_starts, all_txs)
        };
        Ok(Vecs {
            coinjoin: pattern("is_coinjoin", "coinjoin_tx")?,
            consolidation: pattern("is_consolidation", "consolidation_tx")?,
            batch_payout: pattern("is_batch_payout", "batch_payout_tx")?,
        })
    }
}

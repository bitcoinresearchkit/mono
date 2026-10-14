use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::Vecs;
use crate::flagged::Classified;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let pattern =
            |flag, count| Classified::import(db, flag, count, version, mappings, window_starts);
        Ok(Vecs {
            coinjoin: pattern("is_coinjoin", "coinjoin_tx_count")?,
            consolidation: pattern("is_consolidation", "consolidation_tx_count")?,
            batch_payout: pattern("is_batch_payout", "batch_payout_tx_count")?,
        })
    }
}

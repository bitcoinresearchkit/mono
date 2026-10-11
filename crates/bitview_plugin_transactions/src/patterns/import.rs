use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, EagerVec, ImportableVec};

use super::Vecs;
use crate::flagged::Flagged;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let pattern = |flag, count| -> Result<_> {
            Ok(Flagged {
                flag: EagerVec::import(db, flag, version)?,
                count: PerBlockCumulativeRolling::import(
                    db,
                    count,
                    version,
                    mappings,
                    window_starts,
                )?,
            })
        };
        Ok(Vecs {
            coinjoin: pattern("is_coinjoin", "coinjoin_tx_count")?,
            consolidation: pattern("is_consolidation", "consolidation_tx_count")?,
            batch_payout: pattern("is_batch_payout", "batch_payout_tx_count")?,
        })
    }
}

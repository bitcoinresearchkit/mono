use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, EagerVec, ImportableVec};

use super::{CountVecs, Flags, Vecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let count =
            |name| PerBlockCumulativeRolling::import(db, name, version, mappings, window_starts);
        Ok(Vecs {
            count: CountVecs {
                coinjoin: count("coinjoin_tx_count")?,
                consolidation: count("consolidation_tx_count")?,
                batch_payout: count("batch_payout_tx_count")?,
            },
            flags: Flags {
                is_coinjoin: EagerVec::import(db, "is_coinjoin", version)?,
                is_consolidation: EagerVec::import(db, "is_consolidation", version)?,
                is_batch_payout: EagerVec::import(db, "is_batch_payout", version)?,
            },
        })
    }
}

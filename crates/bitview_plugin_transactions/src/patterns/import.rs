use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, EagerVec, ImportableVec};

use super::{CountVecs, Flags, Vecs};

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let count = |name| {
            PerBlockCumulativeRolling::forced_import(db, name, version, mappings, window_starts)
        };
        Ok(Vecs {
            count: CountVecs {
                coinjoin: count("coinjoin_count")?,
                consolidation: count("consolidation_count")?,
                batch_payout: count("batch_payout_count")?,
            },
            flags: Flags {
                is_coinjoin: EagerVec::forced_import(db, "is_coinjoin", version)?,
                is_consolidation: EagerVec::forced_import(db, "is_consolidation", version)?,
                is_batch_payout: EagerVec::forced_import(db, "is_batch_payout", version)?,
            },
        })
    }
}

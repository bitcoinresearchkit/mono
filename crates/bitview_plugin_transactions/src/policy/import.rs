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
        Ok(Vecs {
            nonstandard: Flagged {
                flag: EagerVec::import(db, "is_nonstandard", version)?,
                count: PerBlockCumulativeRolling::import(
                    db,
                    "nonstandard_tx_count",
                    version,
                    mappings,
                    window_starts,
                )?,
            },
        })
    }
}

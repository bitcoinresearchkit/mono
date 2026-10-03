use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, EagerVec, ImportableVec};

use super::{CountVecs, Vecs};

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Vecs {
            count: CountVecs {
                nonstandard: PerBlockCumulativeRolling::forced_import(
                    db,
                    "nonstandard_count",
                    version,
                    mappings,
                    window_starts,
                )?,
            },
            is_nonstandard: EagerVec::forced_import(db, "is_nonstandard", version)?,
        })
    }
}

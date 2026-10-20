use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, ReadableCloneableVec};

use super::Vecs;
use crate::flagged::TxCount;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        all_txs: &impl ReadableCloneableVec<Height, Count>,
    ) -> Result<Self> {
        let import = |kind| {
            TxCount::import(
                db,
                kind,
                version + Version::ONE,
                mappings,
                window_starts,
                all_txs,
            )
        };
        Ok(Vecs {
            v1: import("v1_tx")?,
            v2: import("v2_tx")?,
            v3: import("v3_tx")?,
            other: import("other_version_tx")?,
        })
    }
}

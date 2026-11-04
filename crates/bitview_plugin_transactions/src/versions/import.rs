use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Height, Sats, Version};
use vecdb::{Database, ReadableCloneableVec};

use super::Vecs;
use crate::flagged::{Totals, TxCount};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        totals: Totals<
            '_,
            impl ReadableCloneableVec<Height, Count>,
            impl ReadableCloneableVec<Height, Sats>,
        >,
    ) -> Result<Self> {
        let import = |kind| {
            TxCount::import(
                db,
                kind,
                version + Version::ONE,
                mappings,
                window_starts,
                totals,
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

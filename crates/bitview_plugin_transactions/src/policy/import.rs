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
        Ok(Vecs {
            nonstandard: Classified::import(
                db,
                "is_nonstandard",
                "nonstandard_tx_count",
                version + Version::ONE,
                mappings,
                window_starts,
            )?,
        })
    }
}

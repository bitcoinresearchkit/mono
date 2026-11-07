use bitview_plugin::ImportContext;
use brk_error::Result;
use vecdb::{BytesVec, ImportableVec, PcoVec};

use crate::{STORAGE, Vecs};

impl Vecs {
    pub fn import(context: ImportContext<'_>) -> Result<Self> {
        let db = STORAGE.open_database(context, 10_000_000)?;
        let version = STORAGE.schema_version();
        let this = Self {
            end_tweak_index: PcoVec::import(&db, "end_tweak_index", version)?,
            tx_index: PcoVec::import(&db, "tweak_tx_index", version)?,
            tweak_parity: PcoVec::import(&db, "tweak_parity", version)?,
            tweak_x: BytesVec::import(&db, "tweak_x", version)?,
            end_new_outputs_filter_index: PcoVec::import(
                &db,
                "end_new_outputs_filter_index",
                version,
            )?,
            new_outputs_filter: BytesVec::import(&db, "new_outputs_filter", version)?,
            end_spent_outputs_filter_index: PcoVec::import(
                &db,
                "end_spent_outputs_filter_index",
                version,
            )?,
            spent_outputs_filter: BytesVec::import(&db, "spent_outputs_filter", version)?,
            db,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

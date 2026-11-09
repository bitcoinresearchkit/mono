use bitview_plugin::ImportContext;
use brk_error::Result;
use vecdb::{BytesVec, ImportableVec, PcoVec};

use crate::{STORAGE, Vecs};

impl Vecs {
    pub fn import(context: ImportContext<'_>) -> Result<Self> {
        STORAGE.import(context, 10_000_000, |db, version| {
            let this = Self {
                end_tweak_index: PcoVec::import(&db, "end_tweak_index", version)?,
                tx_index: PcoVec::import(&db, "tweak_tx_index", version)?,
                tweak_parity: PcoVec::import(&db, "tweak_parity", version)?,
                tweak_x: BytesVec::import(&db, "tweak_x", version)?,
                has_off_curve_key: PcoVec::import(&db, "has_off_curve_key", version)?,
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
            Ok(this)
        })
    }
}

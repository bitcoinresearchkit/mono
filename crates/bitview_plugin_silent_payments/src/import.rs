use bitview_plugin::ImportContext;
use brk_error::Result;
use vecdb::{BytesVec, ImportableVec, PcoVec};

use crate::{STORAGE, Vecs};

impl Vecs {
    pub fn import(context: ImportContext<'_>) -> Result<Self> {
        let db = STORAGE.open_database(context, 10_000_000)?;
        let version = STORAGE.schema_version();
        let this = Self {
            first_tweak_index: PcoVec::import(&db, "first_tweak_index", version)?,
            tx_index: PcoVec::import(&db, "tweak_tx_index", version)?,
            tweak: BytesVec::import(&db, "tweak", version)?,
            db,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

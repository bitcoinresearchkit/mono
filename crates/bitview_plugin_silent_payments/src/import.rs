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
            tweak_parity: PcoVec::import(&db, "tweak_parity", version)?,
            tweak_x: BytesVec::import(&db, "tweak_x", version)?,
            first_taproot_prefix_index: PcoVec::import(&db, "first_taproot_prefix_index", version)?,
            taproot_prefix: BytesVec::import(&db, "taproot_prefix", version)?,
            db,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

use bitview_plugin::ImportContext;
use brk_error::Result;
use vecdb::{EagerVec, ImportableVec};

use super::{STORAGE, Vecs};

impl Vecs {
    pub fn import(context: ImportContext<'_>) -> Result<Self> {
        let db = STORAGE.open_database(context, 1)?;
        let streak =
            EagerVec::forced_import(&db, "near_full_block_streak", STORAGE.schema_version())?;

        let this = Self {
            gate: Default::default(),
            db,
            streak,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

use bitview_plugin::ImportContext;
use brk_error::Result;
use vecdb::{ImportableVec, PcoVec};

use super::{STORAGE, Vecs};

impl Vecs {
    pub(crate) fn import(context: ImportContext<'_>) -> Result<Self> {
        STORAGE.import(context, 1, |db, version| {
            let streak = PcoVec::import(&db, "near_full_block_streak", version)?;

            let this = Self { db, streak };
            Ok(this)
        })
    }
}

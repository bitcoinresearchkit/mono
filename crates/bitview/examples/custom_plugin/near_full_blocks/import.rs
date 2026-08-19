use std::path::Path;

use bitview_compute::db_utils::{finalize_db, open_db};
use bitview_plugin::PLUGIN_DATA_DIR;
use brk_error::Result;
use brk_types::Version;
use vecdb::{EagerVec, ImportableVec};

use super::{ID, Vecs};

const VERSION: Version = Version::new(1);

impl Vecs {
    pub fn forced_import(outputs_path: &Path) -> Result<Self> {
        let plugins_path = outputs_path.join(PLUGIN_DATA_DIR);
        let db = open_db(&plugins_path, ID.as_str(), 1)?;
        let streak = EagerVec::forced_import(&db, "near_full_block_streak", VERSION)?;

        let this = Self {
            gate: Default::default(),
            db,
            streak,
        };
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}

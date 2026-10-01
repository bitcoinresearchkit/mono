use std::path::PathBuf;

use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_vecs::{PerBlock, import_cached};
use brk_error::Result;
use statedb::History;

use crate::{STORAGE, Vecs};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &Mappings,
        spends_path: PathBuf,
        creations_path: PathBuf,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();
        let this = Self {
            supply: import_cached(&db, "unspent_sats", version)?,
            count: PerBlock::forced_import(&db, "utxo_count_bis", version, mappings)?,
            history: History::open(db.path())?,
            spends_path,
            creations_path,
            db,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

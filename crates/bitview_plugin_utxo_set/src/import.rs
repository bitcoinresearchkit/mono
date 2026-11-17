use std::path::PathBuf;

use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_urpd::UrpdHistory;
use bitview_vecs::{LazyPerBlock, PerBlock, import_cached};
use brk_error::Result;
use brk_types::Version;
use statedb::History;
use vecdb::Convert;

use crate::{STORAGE, URPD, Vecs};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &Mappings,
        spends_path: PathBuf,
        creations_path: PathBuf,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();
        let supply = import_cached(&db, "unspent_sats", version)?;
        let circulating_supply = LazyPerBlock::from_height_source::<Convert>(
            "circulating_supply",
            version,
            &supply,
            mappings,
        );
        let history = History::open(db.path())?;
        let this = Self {
            supply,
            circulating_supply,
            count: PerBlock::import(&db, "utxo_count", version + Version::ONE, mappings)?,
            live: history.live(),
            history,
            urpd: UrpdHistory::open(&db.path().join(URPD))?,
            spends_path,
            creations_path,
            db,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{OhlcPrice, SplitPrice, SpotPrice};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use crate::{STORAGE, Vecs};

impl Vecs {
    pub fn import(context: ImportContext<'_>, mappings: &MappingsVecs) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let this = Self::import_inner(&db, STORAGE.schema_version(), mappings)?;
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }

    fn import_inner(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let spot = SpotPrice::import(db, "price", version, mappings)?;
        let ohlc = OhlcPrice::from_spot("price_ohlc", version, mappings, &spot);
        let split = SplitPrice::new("price", version, mappings, &spot, &ohlc);

        Ok(Self {
            db: db.clone(),
            split,
            ohlc,
            spot,
        })
    }
}

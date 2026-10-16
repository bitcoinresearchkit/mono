use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyPerBlock, Price, ops::CentsUnsignedToSats};
use brk_error::Result;
use vecdb::{ImportableVec, PcoVec, ReadableCloneableVec};

use crate::{STORAGE, Vecs, ohlc::OhlcPrice, split::SplitPrice};

impl Vecs {
    pub fn import(context: ImportContext<'_>, mappings: &MappingsVecs) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let version = STORAGE.schema_version();
        let spot = Price::import(&db, "price", version, mappings)?;
        let prices = spot.cents.height.read_only_boxed_clone();
        let ohlc = OhlcPrice::new("price_ohlc", version, mappings, &prices);
        let split = SplitPrice::new("price", version, mappings, &prices, &ohlc);
        let sats_per_dollar = LazyPerBlock::from_resolutions::<CentsUnsignedToSats>(
            "sats_per_dollar",
            version,
            &spot.cents,
        );
        let oracle_bin = PcoVec::import(&db, "oracle_bin", version)?;

        let this = Self {
            db,
            split,
            ohlc,
            spot,
            sats_per_dollar,
            oracle_bin,
            oracle: None,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

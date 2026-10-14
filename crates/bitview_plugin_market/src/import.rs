use bitview_plugin::ImportContext;
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use brk_error::Result;

use super::{STORAGE, Vecs, ath, moving_average, range, returns};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        blocks: &BlocksVecs,
        prices: &PriceVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 250_000)?;
        let version = STORAGE.schema_version();

        let spot_price = prices.spot.cents.resolutions.height_source();
        let ath = ath::Vecs::import(&db, version, mappings, spot_price)?;
        let returns = returns::import(version, mappings, &blocks.lookback, prices);
        let range = range::Vecs::import(&db, version, mappings, spot_price)?;
        let moving_average = moving_average::Vecs::import(&db, version, mappings, blocks)?;

        let this = Self {
            db,
            ath,
            returns,
            range,
            moving_average,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

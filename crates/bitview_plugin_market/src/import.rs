use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use brk_error::Result;

use super::{STORAGE, Vecs, ath, moving_average, range, returns};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        prices: &PriceVecs,
    ) -> Result<Self> {
        STORAGE.import(context, 250_000, |db, version| {
            let spot_price = prices.spot.cents.resolutions.height_source();
            let ath = ath::Vecs::import(&db, version, mappings, spot_price)?;
            let returns = returns::import(version, mappings, prices);
            let range = range::Vecs::import(&db, version, mappings)?;
            let moving_average = moving_average::Vecs::import(&db, version, mappings)?;

            Ok(Self {
                db,
                ath,
                returns,
                range,
                moving_average,
            })
        })
    }
}

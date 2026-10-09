use bitview_collections::ByLookbackPeriod;
use bitview_plugin::ImportContext;
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use brk_error::{Error, Result};

use super::{STORAGE, Vecs, ath, lookback, moving_average, range, returns, technical, volatility};

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
        let window_starts = ByLookbackPeriod::try_new(|_, days| {
            Ok::<_, Error>(blocks.lookback.start_vec(days as usize))
        })?;
        let lookback = lookback::Vecs::new(version, mappings, &window_starts, prices)?;
        let returns = returns::Vecs::import(&db, version, mappings, &window_starts, prices)?;
        let volatility = volatility::Vecs::new(version, &returns);
        let range = range::Vecs::import(&db, version, mappings, spot_price)?;
        let moving_average =
            moving_average::Vecs::import(&db, version, mappings, blocks, spot_price)?;
        let technical =
            technical::Vecs::import(&db, version, mappings, &returns.periods._24h.percent)?;

        let this = Self {
            db,
            ath,
            lookback,
            returns,
            volatility,
            range,
            moving_average,
            technical,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

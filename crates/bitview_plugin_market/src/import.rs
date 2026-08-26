use brk_error::Result;

use brk_error::Error;
use vecdb::ReadOnlyClone;

use bitview_compute::ByLookbackPeriod;
use bitview_plugin::ImportContext;

use super::{STORAGE, Vecs};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &bitview_plugin_mappings::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        prices: &bitview_plugin_price::Vecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 250_000)?;
        let version = STORAGE.schema_version();

        let spot_price = prices.spot.cents.height.read_only_cached_boxed_clone();
        let ath = super::ath::forced_import(&db, version, mappings, &spot_price)?;
        let cached_starts = ByLookbackPeriod::try_new(|_, days| {
            Ok::<_, Error>(
                blocks
                    .lookback
                    .cached_start_vec(days as usize)
                    .read_only_cached_boxed_clone(),
            )
        })?;
        let lookback = super::lookback::forced_import(version, mappings, &cached_starts, prices)?;
        let returns =
            super::returns::forced_import(&db, version, mappings, &cached_starts, prices)?;
        let volatility = super::volatility::forced_import(version, &returns)?;
        let cached_spot_price = prices.spot.cents.height.read_only_clone();
        let range = super::range::forced_import(&db, version, mappings, &cached_spot_price)?;
        let moving_average =
            super::moving_average::forced_import(&db, version, mappings, blocks, &spot_price)?;
        let technical =
            super::technical::forced_import(&db, version, mappings, &returns.periods._24h.ratio)?;

        let this = Self {
            plugin_gate: Default::default(),
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

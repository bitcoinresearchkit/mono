use brk_error::Result;

use std::path::Path;

use brk_error::Error;
use brk_types::Version;
use vecdb::ReadOnlyClone;

use bitview_compute::{
    ByLookbackPeriod,
    db_utils::{finalize_db, open_db},
};

use super::Vecs;

impl Vecs {
    pub fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        prices: &bitview_plugin_price::Vecs,
    ) -> Result<Self> {
        let db = open_db(parent_path, super::DB_NAME, 250_000)?;
        let version = parent_version;

        let spot_price = prices.spot.cents.height.read_only_cached_boxed_clone();
        let ath = super::ath::forced_import(&db, version, indexes, &spot_price)?;
        let cached_starts = ByLookbackPeriod::try_new(|_, days| {
            Ok::<_, Error>(
                blocks
                    .lookback
                    .cached_start_vec(days as usize)
                    .read_only_cached_boxed_clone(),
            )
        })?;
        let lookback = super::lookback::forced_import(version, indexes, &cached_starts, prices)?;
        let returns = super::returns::forced_import(&db, version, indexes, &cached_starts, prices)?;
        let volatility = super::volatility::forced_import(version, &returns)?;
        let cached_spot_price = prices.spot.cents.height.read_only_clone();
        let range = super::range::forced_import(&db, version, indexes, &cached_spot_price)?;
        let moving_average =
            super::moving_average::forced_import(&db, version, indexes, blocks, &spot_price)?;
        let technical =
            super::technical::forced_import(&db, version, indexes, &returns.periods._24h.ratio)?;

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
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}

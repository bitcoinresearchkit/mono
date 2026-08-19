use brk_error::Result;

use std::path::Path;

use brk_types::{Cents, Version};

use bitview_compute::{
    CachedWindowStartVec, PerBlock, Windows,
    db_utils::{finalize_db, open_db},
};
use bitview_plugin_distribution::AllChainSources;

use super::Vecs;

impl Vecs {
    pub fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        prices: &bitview_plugin_price::Vecs,
        subsidy_cents: &PerBlock<Cents>,
        all_chain: &AllChainSources,
    ) -> Result<Self> {
        let db = open_db(parent_path, crate::ID.as_str(), 250_000)?;
        let version = parent_version;
        let v1 = version + Version::ONE;
        let spot_price = prices.spot.cents.height.read_only_cached_boxed_clone();
        let activity = super::activity::forced_import(&db, version, indexes, cached_starts)?;
        let age_range =
            super::age_range::forced_import(&db, version, indexes, cached_starts, &spot_price)?;
        let supply =
            super::supply::forced_import(&db, v1, indexes, &spot_price, &activity, all_chain)?;
        let aggregate = super::aggregate::forced_import(
            &db,
            version + Version::new(4),
            indexes,
            &spot_price,
            &supply.active_supply_in_loss_share,
        )?;
        let value = super::value::forced_import(&db, v1, indexes, cached_starts)?;
        let cap = super::cap::forced_import(&db, version + Version::TWO, indexes, subsidy_cents)?;
        let prices = super::prices::forced_import(
            &db,
            version + Version::new(3),
            indexes,
            &spot_price,
            all_chain,
            &cap.cointime.cents.height,
        )?;
        let adjusted = super::adjusted::forced_import(&db, version, indexes)?;
        let reserve_risk = super::reserve_risk::forced_import(&db, v1, indexes, &spot_price)?;

        let this = Self {
            plugin_gate: Default::default(),
            db,
            activity,
            age_range,
            aggregate,
            supply,
            value,
            cap,
            prices,
            adjusted,
            reserve_risk,
        };
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}

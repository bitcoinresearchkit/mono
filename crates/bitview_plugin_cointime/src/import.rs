use brk_error::Result;

use brk_types::{Cents, Version};

use bitview_compute::{CachedWindowStartVec, PerBlock, Windows};
use bitview_plugin::ImportContext;
use bitview_plugin_distribution::AllChainSources;

use super::{STORAGE, Vecs};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        prices: &bitview_plugin_price::Vecs,
        subsidy_cents: &PerBlock<Cents>,
        all_chain: &AllChainSources,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 250_000)?;
        let version = STORAGE.schema_version();
        let v1 = version + Version::ONE;
        let spot_price = prices.spot.cents.height.read_only_cached_boxed_clone();
        let activity = super::activity::forced_import(&db, version, mappings, cached_starts)?;
        let age_range =
            super::age_range::forced_import(&db, version, mappings, cached_starts, &spot_price)?;
        let supply =
            super::supply::forced_import(&db, v1, mappings, &spot_price, &activity, all_chain)?;
        let aggregate = super::aggregate::forced_import(
            &db,
            version + Version::new(4),
            mappings,
            &spot_price,
            &supply.active_supply_in_loss_share,
        )?;
        let value = super::value::forced_import(&db, v1, mappings, cached_starts)?;
        let cap = super::cap::forced_import(&db, version + Version::TWO, mappings, subsidy_cents)?;
        let prices = super::prices::forced_import(
            &db,
            version + Version::new(3),
            mappings,
            &spot_price,
            all_chain,
            &cap.cointime.cents.height,
        )?;
        let adjusted = super::adjusted::forced_import(&db, version, mappings)?;
        let reserve_risk = super::reserve_risk::forced_import(&db, v1, mappings, &spot_price)?;

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
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

use bitview_collections::Windows;
use bitview_distribution::AllChainSources;
use bitview_plugin::ImportContext;
use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlock};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::ReadableCloneableVec;

use super::{
    STORAGE, Vecs, activity, adjusted, age_range, aggregate, cap, prices, reserve_risk, supply,
    value,
};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        prices: &PriceVecs,
        subsidy_cents: &PerBlock<Cents>,
        all_chain: &AllChainSources,
        age: &AgeVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 250_000)?;
        let version = STORAGE.schema_version();
        let v1 = version + Version::ONE;
        let spot_price = prices.spot.cents.height.read_only_boxed_clone();
        let activity = activity::Vecs::import(&db, version, mappings, window_starts, age)?;
        let age_ranges =
            age_range::Vecs::import(&db, version, mappings, window_starts, &spot_price, age)?;
        let supply = supply::Vecs::new(v1, mappings, &spot_price, &activity, all_chain);
        let aggregate =
            aggregate::Vecs::import(&db, version + Version::new(4), mappings, &spot_price)?;
        let value = value::Vecs::import(&db, v1, mappings, window_starts)?;
        let caps = cap::Vecs::import(
            &db,
            version + Version::TWO,
            mappings,
            subsidy_cents,
            &supply,
            all_chain.realized_cap(),
        )?;
        let prices = prices::Vecs::import(
            &db,
            version + Version::new(3),
            mappings,
            all_chain,
            caps.cointime.cents.resolutions.height_source(),
        )?;
        let adjusted = adjusted::Vecs::import(&db, version, mappings)?;
        let reserve_risk =
            reserve_risk::Vecs::import(&db, v1, mappings, window_starts, &spot_price)?;

        let this = Self {
            db,
            urpd_replay: Default::default(),
            activity,
            age_ranges,
            aggregate,
            supply,
            value,
            caps,
            prices,
            adjusted,
            reserve_risk,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}

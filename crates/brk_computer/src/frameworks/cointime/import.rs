use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::Database;

use crate::{distribution::AllChainSources, indexes, internal::PerBlock, price};

use super::{
    ActivityVecs, AdjustedVecs, AgeRangeVecs, AggregateVecs, CapVecs, PricesVecs, ReserveRiskVecs,
    SupplyVecs, ValueVecs, Vecs,
};

use crate::internal::{CachedWindowStartVec, Windows};

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        parent_version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        prices: &price::Vecs,
        subsidy_cents: &PerBlock<Cents>,
        all_chain: &AllChainSources,
    ) -> Result<Self> {
        let version = parent_version;
        let v1 = version + Version::ONE;
        let spot_price = prices.spot.cents.height.read_only_cached_boxed_clone();
        let activity = ActivityVecs::forced_import(db, version, indexes, cached_starts)?;
        let age_range =
            AgeRangeVecs::forced_import(db, version, indexes, cached_starts, &spot_price)?;
        let supply = SupplyVecs::forced_import(db, v1, indexes, &spot_price, &activity, all_chain)?;
        let aggregate = AggregateVecs::forced_import(
            db,
            version + Version::new(4),
            indexes,
            &spot_price,
            &supply.active_supply_in_loss_share,
        )?;
        let value = ValueVecs::forced_import(db, v1, indexes, cached_starts)?;
        let cap = CapVecs::forced_import(db, version + Version::TWO, indexes, subsidy_cents)?;
        let prices = PricesVecs::forced_import(
            db,
            version + Version::new(3),
            indexes,
            &spot_price,
            all_chain,
            &cap.cointime.cents.height,
        )?;
        let adjusted = AdjustedVecs::forced_import(db, version, indexes)?;
        let reserve_risk = ReserveRiskVecs::forced_import(db, v1, indexes, &spot_price)?;

        Ok(Self {
            activity,
            age_range,
            aggregate,
            supply,
            value,
            cap,
            prices,
            adjusted,
            reserve_risk,
        })
    }
}

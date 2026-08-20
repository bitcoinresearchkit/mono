use brk_error::Result;

use brk_types::{Cents, Height, PartsPerMillionSigned32, Version};
use vecdb::{BinaryTransform, CachedBoxedVec, Database, ReadableCloneableVec};

use super::Vecs;
use bitview_compute::{
    CACHE_BUDGET, DaysToYears, LazyIndexedVec, LazyPerBlock, LazyPercentPerBlock, PerBlock, Price,
    RatioDiffCents,
};

const VERSION: Version = Version::ONE;

pub fn forced_import(
    db: &Database,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
    spot_price: &CachedBoxedVec<Height, Cents>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, mappings, spot_price)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let v = version + VERSION;

        let high = Price::forced_import(db, "price_ath", v, mappings)?;

        let max_days_between =
            PerBlock::forced_import(db, "max_days_between_price_ath", v, mappings)?;

        let max_years_between = LazyPerBlock::from_computed::<DaysToYears>(
            "max_years_between_price_ath",
            v,
            max_days_between.height.read_only_boxed_clone(),
            &max_days_between,
        );

        let days_since = PerBlock::forced_import(db, "days_since_price_ath", v, mappings)?;

        let years_since = LazyPerBlock::from_computed::<DaysToYears>(
            "years_since_price_ath",
            v,
            days_since.height.read_only_boxed_clone(),
            &days_since,
        );

        let drawdown_source = LazyIndexedVec::new(
            "price_drawdown_ppm_source",
            v,
            high.cents.height.read_only_boxed_clone(),
            spot_price.clone(),
            |_, high, spot| RatioDiffCents::<PartsPerMillionSigned32>::apply(spot, high),
        );
        let drawdown_source = CACHE_BUDGET.wrap(drawdown_source);
        let drawdown =
            LazyPercentPerBlock::from_height_source("price_drawdown", v, drawdown_source, mappings);

        Ok(Self {
            high,
            drawdown,
            days_since,
            years_since,
            max_days_between,
            max_years_between,
        })
    }
}

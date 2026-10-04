use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PartsPerMillionSigned32;
use bitview_transforms::{DaysToYears, RatioDiffCents};
use bitview_vecs::{LazyFixedRatioPerBlock, LazyIndexedVec, LazyPerBlock, PerBlock, Price};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{BinaryTransform, Database, ReadableCloneableVec};

use super::{Vecs, seconds_to_days::SecondsToDays};

const VERSION: Version = Version::TWO;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let v = version + VERSION;

        let high = Price::import(db, "price_ath", v, mappings)?;

        let max_days_between = PerBlock::import(db, "max_days_between_price_ath", v, mappings)?;

        let max_years_between = LazyPerBlock::from_resolutions::<DaysToYears>(
            "max_years_between_price_ath",
            v,
            &max_days_between,
        );

        let seconds_since = PerBlock::import(db, "seconds_since_price_ath", v, mappings)?;
        let days_since = LazyPerBlock::from_resolutions::<SecondsToDays>(
            "days_since_price_ath",
            v,
            &seconds_since,
        );

        let years_since =
            LazyPerBlock::from_lazy::<DaysToYears, _>("years_since_price_ath", v, &days_since);

        let drawdown_source = LazyIndexedVec::new(
            "price_drawdown_ppm_source",
            v,
            high.cents.resolutions.height_source(),
            spot_price,
            |_, high, spot| RatioDiffCents::<PartsPerMillionSigned32>::apply(spot, high),
        );
        let drawdown = LazyFixedRatioPerBlock::from_height_source(
            "price_drawdown",
            v,
            &drawdown_source,
            mappings,
        );

        Ok(Vecs {
            high,
            drawdown,
            seconds_since,
            days_since,
            years_since,
            max_days_between,
            max_years_between,
        })
    }
}

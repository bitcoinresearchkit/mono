use brk_error::{Error, Result};
use brk_types::{Dollars, Height, PartsPerMillionSigned64, Version};
use vecdb::{BinaryTransform, CachedBoxedVec, Database, ReadableCloneableVec};

use super::super::lookback::ByLookbackPeriod;
use super::Vecs;
use crate::{
    indexes,
    internal::{
        CACHE_BUDGET, LazyPercentPerBlock, LazyWindowVec, RatioDiffDollars, StdDevPerBlock, Windows,
    },
    investing::{ByDcaCagr, ByDcaPeriod},
    price,
};

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &ByLookbackPeriod<CachedBoxedVec<Height, Height>>,
        prices: &price::Vecs,
    ) -> Result<Self> {
        let periods =
            ByLookbackPeriod::try_from_period(cached_starts, |name, _days, window_starts| {
                let metric_name = format!("price_return_{name}");
                let source = LazyWindowVec::<Height, Dollars, PartsPerMillionSigned64>::new(
                    &format!("{metric_name}_ppm_source"),
                    version,
                    prices.spot.usd.height.read_only_boxed_clone(),
                    window_starts.clone(),
                    false,
                    |current, past, _| {
                        RatioDiffDollars::<PartsPerMillionSigned64>::apply(current, past)
                    },
                );
                let source = CACHE_BUDGET.wrap(source);
                Ok::<_, Error>(LazyPercentPerBlock::from_height_source(
                    &metric_name,
                    version,
                    source,
                    indexes,
                ))
            })?;

        let dca_periods = ByDcaPeriod::from_lookback(&periods);
        let cagr = ByDcaCagr::try_new(&dca_periods, |name, days, source| {
            Ok::<_, Error>(LazyPercentPerBlock::from_lazy_cagr(
                &format!("price_cagr_{name}"),
                version,
                (days / 365) as u8,
                source,
            ))
        })?;

        let mut days_iter = Windows::<()>::DAYS.iter();
        let sd_24h = Windows::try_from_fn(|suffix| {
            let days = *days_iter.next().unwrap();
            StdDevPerBlock::forced_import(
                db,
                "price_return_24h",
                suffix,
                days,
                version + Version::ONE,
                indexes,
            )
        })?;

        Ok(Self {
            periods,
            cagr,
            sd_24h,
        })
    }
}

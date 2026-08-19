use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Cents, Dollars, Height, PartsPerMillion64, SatsFract, StoredF32, Version};
use vecdb::{CachedBoxedVec, Database, ReadableCloneableVec, Rw, StorageMode, unlikely};

use crate::{CACHE_BUDGET, LazyIndexedVec, LazyPerBlock, LazyRatioPerBlock, PerBlock, Price};

#[derive(Traversable)]
pub struct PriceWithRatioPerBlock<M: StorageMode = Rw> {
    /// Reported in USD per BTC.
    pub usd: LazyPerBlock<Dollars, Cents>,
    /// Reported in cents per BTC.
    pub cents: PerBlock<Cents, M>,
    /// Reported in sats per USD: 100,000,000 divided by the price in USD per BTC.
    pub sats: LazyPerBlock<SatsFract, Dollars>,
    /// Spot price divided by this price in parts per million; 1,000,000
    /// represents a ratio of 1.0.
    pub ppm: LazyPerBlock<PartsPerMillion64>,
    /// Spot price divided by this price as a unitless decimal ratio.
    pub ratio: LazyPerBlock<StoredF32, PartsPerMillion64>,
}

impl PriceWithRatioPerBlock {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let price = Price::forced_import(db, name, version, indexes)?;
        let ratio_version = version + Version::new(4);
        let ppm_source = LazyIndexedVec::new(
            &format!("{name}_ratio_ppm_source"),
            ratio_version,
            price.cents.height.read_only_boxed_clone(),
            spot_price.clone(),
            |_, price, spot| price_ratio(spot, price),
        );
        let ppm_source = CACHE_BUDGET.wrap(ppm_source);
        let ratio = LazyRatioPerBlock::from_height_source(
            &format!("{name}_ratio"),
            ratio_version,
            ppm_source,
            indexes,
        );
        Ok(Self {
            usd: price.usd,
            cents: price.cents,
            sats: price.sats,
            ppm: ratio.ppm,
            ratio: ratio.ratio,
        })
    }
}

pub fn price_ratio(close: Cents, price: Cents) -> PartsPerMillion64 {
    if unlikely(price == Cents::ZERO) {
        PartsPerMillion64::NAN
    } else {
        PartsPerMillion64::from(f64::from(close) / f64::from(price))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_price_has_no_ratio() {
        assert!(price_ratio(Cents::new(100), Cents::ZERO).is_nan());
        assert_eq!(
            price_ratio(Cents::new(100), Cents::new(50)),
            PartsPerMillion64::from(2.0),
        );
    }
}

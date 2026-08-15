use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Dollars, Height, PartsPerMillion64, SatsFract, StoredF32, Version};
use vecdb::{CachedBoxedVec, Database, ReadableCloneableVec, Rw, StorageMode, unlikely};

use crate::{
    indexes,
    internal::{CACHE_BUDGET, LazyIndexedVec, LazyPerBlock, LazyRatioPerBlock, PerBlock, Price},
};

#[derive(Traversable)]
pub struct PriceWithRatioPerBlock<M: StorageMode = Rw> {
    pub usd: LazyPerBlock<Dollars, Cents>,
    pub cents: PerBlock<Cents, M>,
    pub sats: LazyPerBlock<SatsFract, Dollars>,
    pub ppm: LazyPerBlock<PartsPerMillion64>,
    pub ratio: LazyPerBlock<StoredF32, PartsPerMillion64>,
}

impl PriceWithRatioPerBlock {
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
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

pub(super) fn price_ratio(close: Cents, price: Cents) -> PartsPerMillion64 {
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

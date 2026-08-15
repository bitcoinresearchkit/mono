use brk_traversable::Traversable;
use brk_types::{Cents, Dollars, Height, PartsPerMillion64, SatsFract, StoredF32, Version};
use vecdb::{CachedBoxedVec, ColumnId, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec};

use crate::{
    indexes,
    internal::{
        CACHE_BUDGET, LazyColumnPerBlock, LazyIndexedVec, LazyPerBlock, LazyRatioPerBlock, Price,
    },
};

use super::price::price_ratio;

#[derive(Clone, Traversable)]
pub struct LazyColumnPriceWithRatioPerBlock<C>
where
    C: ColumnId,
{
    pub usd: LazyPerBlock<Dollars, Cents>,
    pub cents: LazyColumnPerBlock<Cents, C>,
    pub sats: LazyPerBlock<SatsFract, Dollars>,
    pub ppm: LazyPerBlock<PartsPerMillion64>,
    pub ratio: LazyPerBlock<StoredF32, PartsPerMillion64>,
}

impl<C> LazyColumnPriceWithRatioPerBlock<C>
where
    C: ColumnId,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, Cents>, C>,
        column: C,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let price = Price::from_columnar_source(name, version, source, column, indexes);
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

        Self {
            usd: price.usd,
            cents: price.cents,
            sats: price.sats,
            ppm: ratio.ppm,
            ratio: ratio.ratio,
        }
    }
}

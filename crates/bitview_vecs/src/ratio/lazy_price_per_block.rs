use brk_types::{Cents, Height, Version};
use vecdb::ReadableCloneableVec;

use crate::{IndexSources, LazyPerBlock, LazyRatioPerBlock, Price, PriceWithRatio};

pub type LazyPriceWithRatioPerBlock = PriceWithRatio<Price<LazyPerBlock<Cents>>>;

impl LazyPriceWithRatioPerBlock {
    pub fn from_height_source<V>(
        name: &str,
        version: Version,
        source: &V,
        indexes: &IndexSources,
        spot_price: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Self
    where
        V: ReadableCloneableVec<Height, Cents> + ?Sized,
    {
        let price = Price::from_height_source(name, version, source, indexes);
        let ratio = LazyRatioPerBlock::from_price_source(
            &format!("{name}_ratio"),
            version,
            &price.cents.height,
            spot_price,
            indexes,
        );

        Self {
            price,
            relative: ratio,
        }
    }
}

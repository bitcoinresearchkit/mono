use brk_traversable::Traversable;
use brk_types::{Cents, Dollars, Height, PartsPerMillion64, SatsFract, StoredF32, Version};
use vecdb::{CachedBoxedVec, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec};

use crate::{
    indexes,
    internal::{Identity, LazyIndexedVec, LazyPerBlock, LazyRatioPerBlock, Price},
};

use super::price::price_ratio;

#[derive(Clone, Traversable)]
pub struct LazyPriceWithRatioPerBlock {
    pub usd: LazyPerBlock<Dollars, Cents>,
    pub cents: LazyPerBlock<Cents>,
    pub sats: LazyPerBlock<SatsFract, Dollars>,
    pub ppm: LazyPerBlock<PartsPerMillion64>,
    pub ratio: LazyPerBlock<StoredF32, PartsPerMillion64>,
}

impl LazyPriceWithRatioPerBlock {
    pub(crate) fn from_boxed_height_source(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, Cents>,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let source = LazyPerBlock::from_uncached_boxed_height_source::<Identity<Cents>>(
            &format!("{name}_cents"),
            version,
            source,
            indexes,
        );
        let price = Price::from_lazy_cents_source::<Identity<Cents>, Cents>(name, version, &source);
        let ratio_version = version + Version::new(4);
        let ppm_source = LazyIndexedVec::new(
            &format!("{name}_ratio_ppm_source"),
            ratio_version,
            price.cents.height.read_only_boxed_clone(),
            spot_price.clone(),
            |_, price, spot| price_ratio(spot, price),
        );
        let ratio = LazyRatioPerBlock::from_uncached_height_source(
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

    pub(crate) fn from_uncached_height_source<V>(
        name: &str,
        version: Version,
        source: V,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self
    where
        V: TypedVec<I = Height, T = Cents> + ReadableVec<Height, Cents> + Clone + 'static,
    {
        let price = Price::from_uncached_height_source(name, version, source, indexes);
        let ratio_version = version + Version::new(4);
        let ppm_source = LazyIndexedVec::new(
            &format!("{name}_ratio_ppm_source"),
            ratio_version,
            price.cents.height.read_only_boxed_clone(),
            spot_price.clone(),
            |_, price, spot| price_ratio(spot, price),
        );
        let ratio = LazyRatioPerBlock::from_uncached_height_source(
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

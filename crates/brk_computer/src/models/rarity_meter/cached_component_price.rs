use brk_types::{Cents, CentsCompact, Height, PartsPerMillion32, Version};
use vecdb::{
    AnyVec, BinaryTransform, CachedReadableVec, CachedVec, LazyVec, ReadableCloneableVec, VecIndex,
};

use crate::{
    indexes,
    internal::{LazyIndexedVec, LazyPerBlock, Price, PriceTimesRatio},
};

#[derive(Clone)]
pub(super) struct CachedComponentPrice {
    cache: CachedVec<LazyVec<Height, CentsCompact, Height, Cents>>,
}

impl CachedComponentPrice {
    pub(super) fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, Cents> + 'static),
    ) -> Self {
        let compact = LazyVec::init(
            &format!("{name}_cached_price"),
            version,
            source.read_only_boxed_clone(),
            |_, value| CentsCompact::from(value),
        );

        Self {
            cache: CachedVec::wrap(compact),
        }
    }

    pub(super) fn price_for_ratio(
        &self,
        name: &str,
        version: Version,
        ratio: &(impl ReadableCloneableVec<Height, PartsPerMillion32> + 'static),
        indexes: &indexes::Vecs,
    ) -> Price<LazyPerBlock<Cents>> {
        let source = LazyIndexedVec::new(
            &format!("{name}_cents_source"),
            version,
            ratio.read_only_boxed_clone(),
            self.cache.cached_boxed_clone(),
            |_, ratio, price| {
                PriceTimesRatio::<PartsPerMillion32>::apply(Cents::from(price), ratio)
            },
        );

        Price::from_height_source(name, version, source, indexes)
    }

    pub(super) fn clear_if_recomputed_from(&self, height: Height) {
        if height.to_usize() < self.cache.len() {
            self.cache.invalidate();
        }
    }
}

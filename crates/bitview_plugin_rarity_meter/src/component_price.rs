use bitview_primitives::PartsPerMillion32;
use bitview_vecs::{IndexSources, LazyIndexedVec, LazyPerBlock, Price, ops::PriceTimesRatio};
use brk_types::{Cents, Height, Version};
use vecdb::{BinaryTransform, ReadableBoxedVec, ReadableCloneableVec, ReadableVec};

/// A component's reference price, which its ratio percentiles multiply into price bands.
#[derive(Clone)]
pub struct ComponentPrice {
    price: ReadableBoxedVec<Height, Cents>,
}

impl ComponentPrice {
    pub fn new(source: &impl ReadableCloneableVec<Height, Cents>) -> Self {
        Self {
            price: source.read_only_boxed_clone(),
        }
    }

    pub fn price_for_ratio(
        &self,
        name: &str,
        version: Version,
        ratio: &impl ReadableCloneableVec<Height, PartsPerMillion32>,
        mappings: &IndexSources,
    ) -> Price<LazyPerBlock<Cents>> {
        let source = LazyIndexedVec::new(
            &format!("{name}_cents_source"),
            version,
            ratio,
            &self.price,
            |_, ratio, price| PriceTimesRatio::<PartsPerMillion32>::apply(price, ratio),
        );

        Price::from_height_source(name, version, &source, mappings)
    }

    pub fn collect_boundary_prices(
        &self,
        ratios: [&impl ReadableVec<Height, PartsPerMillion32>; 10],
        start: usize,
        end: usize,
    ) -> [Vec<Cents>; 10] {
        let prices = self.price.collect_range_at(start, end);
        ratios.map(|ratio| {
            let mut out = Vec::with_capacity(prices.len());
            ratio.for_each_chunk_at(start, start + prices.len(), &mut |at, values| {
                out.extend(
                    values
                        .iter()
                        .zip(&prices[at - start..])
                        .map(|(&ratio, &price)| {
                            PriceTimesRatio::<PartsPerMillion32>::apply(price, ratio)
                        }),
                );
            });
            out
        })
    }
}

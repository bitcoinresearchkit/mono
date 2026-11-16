use bitview_primitives::{Multiple, PriceRatio};
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, LazyPerBlock, LazyRatioPerBlock, Price, ops::Quotient};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};

use crate::Component;

/// A median creation price's component, with the price and spot's ratio to it. The ratio is a
/// lazy quotient: one division per value read, no disk.
#[derive(Traversable)]
pub struct MedianComponent<M: StorageMode = Rw> {
    #[traversable(flatten, rename = "value")]
    price: Price<LazyPerBlock<Cents>>,
    /// Spot price divided by the price described above, computed from cent-precision prices:
    /// null while that price rounds to zero (2010), approximate while prices are a few cents.
    #[traversable(flatten, rename = "ratio")]
    #[traversable(lead = "Its ratio: the price divided by it.")]
    ratio: LazyRatioPerBlock<PriceRatio, Multiple>,
    #[traversable(flatten)]
    pub component: Component<M>,
}

impl MedianComponent {
    /// `price` names the price, its spot ratio and the component's series (its source's id).
    pub fn import(
        db: &Database,
        price: &str,
        version: Version,
        indexes: &IndexSources,
        source: &impl ReadableCloneableVec<Height, Cents>,
        spot: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let ratio = LazyRatioPerBlock::from_ratio::<Cents, Cents, Quotient<PriceRatio>>(
            &format!("{price}_ratio"),
            version,
            spot,
            source,
            indexes,
        );
        Ok(Self {
            price: Price::from_height_source(price, version, source, indexes),
            component: Component::import(
                db,
                price,
                version,
                indexes,
                source,
                &ratio.fixed.height,
            )?,
            ratio,
        })
    }
}

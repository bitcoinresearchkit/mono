use bitview_primitives::{Lengths, PriceRatio};
use bitview_transforms::price_ratio;
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, LazyPerBlock, Price, RatioPerBlock};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use crate::Component;

#[derive(Traversable)]
pub struct MedianComponent<M: StorageMode = Rw> {
    /// Median creation price from the existing weighted cost-basis distribution_age.
    #[traversable(flatten)]
    pub price: Price<LazyPerBlock<Cents>>,
    /// Spot price divided by the median creation price.
    #[traversable(flatten)]
    pub relative: RatioPerBlock<PriceRatio, M>,
    #[traversable(flatten)]
    pub component: Component<M>,
}

impl MedianComponent {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        source: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        Ok(Self {
            price: Price::from_height_source(name, version, source, indexes),
            relative: RatioPerBlock::import(db, name, version, indexes)?,
            component: Component::import(db, name, version, indexes, source)?,
        })
    }

    pub fn compute(
        &mut self,
        starting_lengths: &Lengths,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.relative.ppm.height.compute_transform2(
            starting_lengths.height,
            spot,
            &self.price.cents.height,
            |(height, spot, price, _)| (height, price_ratio(spot, price)),
            exit,
        )?;
        self.component
            .compute(starting_lengths, &self.relative.ratio.height, exit)
    }
}

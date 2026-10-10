use bitview_primitives::PriceRatio;
use bitview_transforms::price_ratio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use crate::{IndexSources, LazyPerBlock, Price, RatioPerBlock};

/// A price, a lazy view of its source, and its stored spot/price ratio.
#[derive(Deref, DerefMut, Traversable)]
pub struct PriceWithRatio<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten, rename = "block")]
    price: Price<LazyPerBlock<Cents>>,
    /// Spot divided by the price.
    #[traversable(flatten, rename = "ratio")]
    pub relative: RatioPerBlock<PriceRatio, M>,
}

impl PriceWithRatio {
    pub fn import<V>(
        db: &Database,
        name: &str,
        version: Version,
        source: &V,
        indexes: &IndexSources,
    ) -> Result<Self>
    where
        V: ReadableCloneableVec<Height, Cents> + ?Sized,
    {
        Ok(Self {
            price: Price::from_height_source(name, version, source, indexes),
            relative: RatioPerBlock::import(db, &format!("{name}_ratio"), version, indexes)?,
        })
    }

    /// Spot divided by the price, stored.
    pub fn compute_ratio(
        &mut self,
        max_from: Height,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.relative.fixed.height.compute_transform2(
            max_from,
            spot,
            &self.price.cents.height,
            |(height, spot, price, _)| (height, price_ratio(spot, price)),
            exit,
        )?;
        Ok(())
    }
}

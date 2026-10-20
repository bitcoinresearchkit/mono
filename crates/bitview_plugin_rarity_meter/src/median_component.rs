use bitview_primitives::Lengths;
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, PriceWithRatio};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use crate::Component;

#[derive(Traversable)]
pub struct MedianComponent<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub price: PriceWithRatio<M>,
    #[traversable(flatten)]
    pub component: Component<M>,
}

impl MedianComponent {
    /// `price` names the price and its spot ratio (its source's id), `name` the component's own
    /// series.
    pub fn import(
        db: &Database,
        price: &str,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        source: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        Ok(Self {
            price: PriceWithRatio::import(db, price, version, source, indexes)?,
            component: Component::import(db, name, version, indexes, source)?,
        })
    }

    pub fn compute(
        &mut self,
        starting_lengths: &Lengths,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.price
            .compute_ratio(starting_lengths.height, spot, exit)?;
        self.component
            .compute(starting_lengths, &self.price.relative.fixed.height, exit)
    }
}

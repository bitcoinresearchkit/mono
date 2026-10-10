use bitview_primitives::{PriceRatio, Ratio};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Ident, ReadableCloneableVec, Rw, StorageMode};

use crate::{IndexSources, LazyPerBlock, PriceWithRatio};

/// A price with its spot ratio, and that ratio again under its MVRV name.
#[derive(Deref, DerefMut, Traversable)]
pub struct PriceWithMvrv<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    price: PriceWithRatio<M>,
    /// MVRV (market value to realized value): spot divided by the price, the same series as the
    /// ratio.
    mvrv: LazyPerBlock<Ratio>,
}

impl PriceWithMvrv {
    pub fn import<V>(
        db: &Database,
        name: &str,
        mvrv_name: &str,
        version: Version,
        source: &V,
        indexes: &IndexSources,
    ) -> Result<Self>
    where
        V: ReadableCloneableVec<Height, Cents> + ?Sized,
    {
        let price = PriceWithRatio::import(db, name, version, source, indexes)?;
        Ok(Self {
            mvrv: LazyPerBlock::from_lazy::<Ident, PriceRatio>(
                mvrv_name,
                version,
                &price.relative.ratio,
            ),
            price,
        })
    }
}

use bitview_primitives::PERCENTILES_LEN;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

use crate::{CachedSeries, IndexSources, LazyPerBlock, Price, import_cached, shapes::ByPercentile};

#[derive(Deref, DerefMut, Traversable)]
pub struct PercentilesVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub(crate) prices: ByPercentile<Price<LazyPerBlock<Cents>>>,
    #[traversable(hidden)]
    stored: ByPercentile<CachedSeries<Height, Cents, M>>,
}

impl PercentilesVecs {
    pub fn import(
        db: &Database,
        prefix: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let version = version + Version::TWO;
        let stored = ByPercentile::try_from_fn(|id| {
            import_cached(db, &format!("{prefix}_{}_cents", id.suffix()), version)
        })?;
        let prices = ByPercentile::from_fn(|id| {
            Price::from_height_source(
                &format!("{prefix}_{}", id.suffix()),
                version,
                stored.select(id),
                indexes,
            )
        });
        Ok(Self { prices, stored })
    }

    pub fn push(&mut self, prices: &[Cents; PERCENTILES_LEN]) {
        for (target, &price) in self.stored.iter_mut().zip(prices) {
            target.push(price);
        }
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .iter_mut()
            .map(|vec| vec as &mut dyn AnyStoredVec)
    }
}

use bitview_collections::ByPercentile;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Height, PERCENTILES_LEN, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

use crate::{CachedSeries, IndexSources, LazyPerBlock, Price, import_cached};

#[derive(Deref, DerefMut, Traversable)]
pub struct PercentilesVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub prices: ByPercentile<Price<LazyPerBlock<Cents>>>,
    #[traversable(hidden)]
    stored: ByPercentile<CachedSeries<Height, Cents, M>>,
}

impl PercentilesVecs {
    pub fn forced_import(
        db: &Database,
        prefix: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let version = version + Version::TWO;
        let stored = ByPercentile::try_from_fn(|id| {
            import_cached(
                db,
                &format!("{prefix}_pct{:02}_cents", id.percentile()),
                version,
            )
        })?;
        let prices = ByPercentile::from_fn(|id| {
            Price::from_height_source(
                &format!("{prefix}_pct{:02}", id.percentile()),
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

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.stored
            .iter_mut()
            .map(|v| v as &mut dyn AnyStoredVec)
            .collect()
    }
}

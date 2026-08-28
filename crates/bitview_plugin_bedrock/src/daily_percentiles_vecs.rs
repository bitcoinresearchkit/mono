use bitview_compute::{ColumnarDailyMetric, DailyMappings, PercentilePrices};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, PERCENTILES_LEN, PercentileId, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::price::LazyColumnPrice;

#[derive(Deref, DerefMut, Traversable)]
pub struct DailyPercentilesVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub prices: ColumnarDailyMetric<
        Cents,
        PercentileId,
        PercentilePrices<LazyColumnPrice<PercentileId>>,
        M,
    >,
}

const VERSION: Version = Version::ONE;

impl DailyPercentilesVecs {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        let version = version + VERSION;
        let prices = ColumnarDailyMetric::forced_import(db, name, version, |source| {
            PercentilePrices::from_fn(|id| {
                LazyColumnPrice::new(
                    &format!("{name}_pct{:02}", id.percentile()),
                    version,
                    source,
                    id,
                    mappings,
                )
            })
        })?;

        Ok(Self { prices })
    }

    #[inline(always)]
    pub fn push(&mut self, percentile_prices: &[Cents; PERCENTILES_LEN]) {
        self.prices.push(*percentile_prices);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.prices.stored_mut()
    }
}

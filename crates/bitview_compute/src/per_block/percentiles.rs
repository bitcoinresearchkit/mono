use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Cents, Version};
pub use brk_types::{PERCENTILES, PERCENTILES_LEN, PercentileId};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use crate::{ColumnarPerBlock, LazyColumnPerBlock, PercentilePrices, Price};

#[derive(Deref, DerefMut, Traversable)]
pub struct PercentilesVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub prices: ColumnarPerBlock<
        Cents,
        PercentileId,
        PercentilePrices<Price<LazyColumnPerBlock<Cents, PercentileId>>>,
        M,
    >,
}

const VERSION: Version = Version::ONE;

impl PercentilesVecs {
    pub fn forced_import(
        db: &Database,
        prefix: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        let version = version + VERSION;
        let prices = ColumnarPerBlock::<Cents, PercentileId, _>::forced_import(
            db,
            &format!("{prefix}_cents"),
            version,
            |source| {
                PercentilePrices::from_fn(|id| {
                    Price::from_columnar_source(
                        &format!("{prefix}_pct{:02}", id.percentile()),
                        version,
                        source,
                        id,
                        indexes,
                    )
                })
            },
        )?;

        Ok(Self { prices })
    }

    /// Push percentile prices (in cents).
    #[inline(always)]
    pub fn push(&mut self, percentile_prices: &[Cents; PERCENTILES_LEN]) {
        self.prices.push(*percentile_prices);
    }

    /// Validate computed versions or reset if mismatched.
    pub fn validate_computed_version_or_reset(&mut self, version: Version) -> Result<()> {
        self.prices.validate_computed_version_or_reset(version)?;
        Ok(())
    }
}

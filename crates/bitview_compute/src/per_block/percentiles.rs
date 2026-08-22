use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Cents, Version};
pub use brk_types::{PERCENTILES, PERCENTILES_LEN, PercentileId};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use crate::{ColumnarPerBlock, LazyColumnPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct PercentilePrices<T> {
    /// Uses the 5th percentile.
    pub pct05: T,
    /// Uses the 10th percentile.
    pub pct10: T,
    /// Uses the 15th percentile.
    pub pct15: T,
    /// Uses the 20th percentile.
    pub pct20: T,
    /// Uses the 25th percentile.
    pub pct25: T,
    /// Uses the 30th percentile.
    pub pct30: T,
    /// Uses the 35th percentile.
    pub pct35: T,
    /// Uses the 40th percentile.
    pub pct40: T,
    /// Uses the 45th percentile.
    pub pct45: T,
    /// Uses the 50th percentile.
    pub pct50: T,
    /// Uses the 55th percentile.
    pub pct55: T,
    /// Uses the 60th percentile.
    pub pct60: T,
    /// Uses the 65th percentile.
    pub pct65: T,
    /// Uses the 70th percentile.
    pub pct70: T,
    /// Uses the 75th percentile.
    pub pct75: T,
    /// Uses the 80th percentile.
    pub pct80: T,
    /// Uses the 85th percentile.
    pub pct85: T,
    /// Uses the 90th percentile.
    pub pct90: T,
    /// Uses the 95th percentile.
    pub pct95: T,
}

impl<T> PercentilePrices<T> {
    fn from_fn(mut f: impl FnMut(PercentileId) -> T) -> Self {
        Self {
            pct05: f(PercentileId::Pct05),
            pct10: f(PercentileId::Pct10),
            pct15: f(PercentileId::Pct15),
            pct20: f(PercentileId::Pct20),
            pct25: f(PercentileId::Pct25),
            pct30: f(PercentileId::Pct30),
            pct35: f(PercentileId::Pct35),
            pct40: f(PercentileId::Pct40),
            pct45: f(PercentileId::Pct45),
            pct50: f(PercentileId::Pct50),
            pct55: f(PercentileId::Pct55),
            pct60: f(PercentileId::Pct60),
            pct65: f(PercentileId::Pct65),
            pct70: f(PercentileId::Pct70),
            pct75: f(PercentileId::Pct75),
            pct80: f(PercentileId::Pct80),
            pct85: f(PercentileId::Pct85),
            pct90: f(PercentileId::Pct90),
            pct95: f(PercentileId::Pct95),
        }
    }
}

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

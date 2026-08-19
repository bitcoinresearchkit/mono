use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Cents, Version};
pub use brk_types::{PERCENTILES, PERCENTILES_LEN, PercentileId};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use crate::{ColumnarPerBlock, LazyColumnPerBlock, Price};

#[derive(Clone, Traversable)]
pub struct PercentilePrices<T> {
    pub pct05: T,
    pub pct10: T,
    pub pct15: T,
    pub pct20: T,
    pub pct25: T,
    pub pct30: T,
    pub pct35: T,
    pub pct40: T,
    pub pct45: T,
    pub pct50: T,
    pub pct55: T,
    pub pct60: T,
    pub pct65: T,
    pub pct70: T,
    pub pct75: T,
    pub pct80: T,
    pub pct85: T,
    pub pct90: T,
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

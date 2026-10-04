//! Generic price wrapper with cents, USD, and sats representations.
//!
//! The field types preserve each family's source, sampling and rounding policy.
//! The default family derives fractional sats from USD; spot candles retain
//! their integer-sats conversion directly from cents.

use bitview_compute::ComputedVecValue;
use bitview_primitives::SatsFract;
use bitview_transforms::{Convert, DollarsToSatsFract};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Dollars, Height, Version};
use schemars::JsonSchema;
use vecdb::{Database, Ident, ReadableCloneableVec, UnaryTransform};

use crate::{IndexSources, LazyPerBlock, PerBlock};

/// Generic price metric with cents, USD, and sats representations.
#[derive(Clone, Traversable)]
pub struct Price<C, U = LazyPerBlock<Dollars, Cents>, S = LazyPerBlock<SatsFract, Dollars>> {
    /// Reported in USD per BTC.
    pub usd: U,
    /// Reported in cents per BTC.
    pub cents: C,
    /// Reported in sats per USD: 100,000,000 divided by the price in USD per BTC.
    pub(crate) sats: S,
}

impl Price<PerBlock<Cents>> {
    /// Import from database: stored cents, lazy USD + sats.
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let cents = PerBlock::import(db, &format!("{name}_cents"), version, indexes)?;
        let usd = LazyPerBlock::from_resolutions::<Convert>(name, version, &cents);
        Ok(Self::from_cents_and_usd(name, version, cents, usd))
    }
}

impl Price<LazyPerBlock<Cents, Cents>> {
    pub fn from_lazy_cents_source<F, S>(
        name: &str,
        version: Version,
        source: &LazyPerBlock<Cents, S>,
    ) -> Self
    where
        F: UnaryTransform<Cents, Cents>,
        S: ComputedVecValue + JsonSchema,
    {
        let cents = LazyPerBlock::from_lazy::<F, S>(&format!("{name}_cents"), version, source);
        let usd = LazyPerBlock::from_lazy::<Convert, Cents>(name, version, &cents);
        Self::from_cents_and_usd(name, version, cents, usd)
    }
}

impl Price<LazyPerBlock<Cents>> {
    pub fn from_height_source<V>(
        name: &str,
        version: Version,
        source: &V,
        indexes: &IndexSources,
    ) -> Self
    where
        V: ReadableCloneableVec<Height, Cents> + ?Sized,
    {
        let cents = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_cents"),
            version,
            source,
            indexes,
        );
        let usd = LazyPerBlock::from_lazy::<Convert, Cents>(name, version, &cents);
        Self::from_cents_and_usd(name, version, cents, usd)
    }
}

impl<C> Price<C> {
    fn from_cents_and_usd(
        name: &str,
        version: Version,
        cents: C,
        usd: LazyPerBlock<Dollars, Cents>,
    ) -> Self {
        let sats = LazyPerBlock::from_lazy::<DollarsToSatsFract, Cents>(
            &format!("{name}_sats"),
            version,
            &usd,
        );
        Self { usd, cents, sats }
    }
}

//! Generic price wrapper: the USD series over its exact cents.
//!
//! The field types preserve each family's source, sampling and rounding policy.

use bitview_compute::ComputedVecValue;
use bitview_transforms::Convert;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Dollars, Height, Version};
use schemars::JsonSchema;
use vecdb::{Database, Ident, ReadableCloneableVec, UnaryTransform};

use crate::{IndexSources, LazyPerBlock, PerBlock};

/// Generic price metric: the series is the USD price, cents its exact storage.
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct Price<C, U = LazyPerBlock<Dollars, Cents>> {
    /// Reported in USD per BTC.
    pub usd: U,
    /// Reported in cents per BTC.
    #[traversable(hidden)]
    pub cents: C,
}

impl Price<PerBlock<Cents>> {
    /// Import from database: stored cents, lazy USD.
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let cents = PerBlock::import(db, &format!("{name}_cents"), version, indexes)?;
        let usd = LazyPerBlock::from_resolutions::<Convert>(name, version, &cents);
        Ok(Self { usd, cents })
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
        Self { usd, cents }
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
        Self { usd, cents }
    }
}

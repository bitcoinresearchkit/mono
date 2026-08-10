//! Generic price wrapper with cents, USD, and sats representations.
//!
//! All prices use this single struct with different cents types.
//! USD is always lazily derived from cents via CentsUnsignedToDollars.
//! Sats is always lazily derived from USD via DollarsToSatsFract.

use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Dollars, Height, SatsFract, Version};
use schemars::JsonSchema;
use vecdb::{
    ColumnId, Database, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec, ReadableVec, TypedVec,
    UnaryTransform,
};

use super::{LazyColumnPerBlock, LazyPerBlock, PerBlock};
use crate::{
    indexes,
    internal::{CentsUnsignedToDollars, ComputedVecValue, DollarsToSatsFract},
};

/// Generic price metric with cents, USD, and sats representations.
#[derive(Clone, Traversable)]
pub struct Price<C> {
    pub usd: LazyPerBlock<Dollars, Cents>,
    pub cents: C,
    pub sats: LazyPerBlock<SatsFract, Dollars>,
}

impl Price<PerBlock<Cents>> {
    /// Import from database: stored cents, lazy USD + sats.
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let cents = PerBlock::forced_import(db, &format!("{name}_cents"), version, indexes)?;
        let usd = LazyPerBlock::from_computed::<CentsUnsignedToDollars>(
            name,
            version,
            cents.height.read_only_boxed_clone(),
            &cents,
        );
        let sats = LazyPerBlock::from_lazy::<DollarsToSatsFract, Cents>(
            &format!("{name}_sats"),
            version,
            &usd,
        );
        Ok(Self { usd, cents, sats })
    }
}

impl<C> Price<LazyColumnPerBlock<Cents, C>>
where
    C: ColumnId,
{
    pub(crate) fn from_columnar_source(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, Cents>, C>,
        column: C,
        indexes: &indexes::Vecs,
    ) -> Self {
        let cents =
            LazyColumnPerBlock::new(&format!("{name}_cents"), version, source, column, indexes);
        let usd = LazyPerBlock::from_resolutions::<CentsUnsignedToDollars>(
            name,
            version,
            cents.height.read_only_boxed_clone(),
            &cents.resolutions,
        );
        let sats = LazyPerBlock::from_lazy::<DollarsToSatsFract, Cents>(
            &format!("{name}_sats"),
            version,
            &usd,
        );

        Self { usd, cents, sats }
    }
}

impl Price<LazyPerBlock<Cents, Cents>> {
    pub(crate) fn from_lazy_cents_source<F, S>(
        name: &str,
        version: Version,
        source: &LazyPerBlock<Cents, S>,
    ) -> Self
    where
        F: UnaryTransform<Cents, Cents>,
        S: ComputedVecValue + JsonSchema,
    {
        let cents = LazyPerBlock::from_lazy::<F, S>(&format!("{name}_cents"), version, source);
        let usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(name, version, &cents);
        let sats = LazyPerBlock::from_lazy::<DollarsToSatsFract, Cents>(
            &format!("{name}_sats"),
            version,
            &usd,
        );
        Self { usd, cents, sats }
    }
}

impl Price<LazyPerBlock<Cents>> {
    pub(crate) fn from_height_source<V>(
        name: &str,
        version: Version,
        source: V,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        V: TypedVec<I = brk_types::Height, T = Cents>
            + ReadableVec<brk_types::Height, Cents>
            + Clone
            + 'static,
    {
        let cents = LazyPerBlock::from_height_source::<crate::internal::Identity<Cents>, _>(
            &format!("{name}_cents"),
            version,
            source,
            indexes,
        );
        let usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(name, version, &cents);
        let sats = LazyPerBlock::from_lazy::<DollarsToSatsFract, Cents>(
            &format!("{name}_sats"),
            version,
            &usd,
        );
        Self { usd, cents, sats }
    }

    pub(crate) fn from_uncached_height_source<V>(
        name: &str,
        version: Version,
        source: V,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        V: TypedVec<I = brk_types::Height, T = Cents>
            + ReadableVec<brk_types::Height, Cents>
            + Clone
            + 'static,
    {
        let cents = LazyPerBlock::from_uncached_height_source::<crate::internal::Identity<Cents>, _>(
            &format!("{name}_cents"),
            version,
            source,
            indexes,
        );
        let usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(name, version, &cents);
        let sats = LazyPerBlock::from_lazy::<DollarsToSatsFract, Cents>(
            &format!("{name}_sats"),
            version,
            &usd,
        );
        Self { usd, cents, sats }
    }
}

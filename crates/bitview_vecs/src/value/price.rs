use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Dollars, Height, Version};
use vecdb::{Convert, Database, Ident, ReadableCloneableVec};

use crate::{IndexSources, LazyPerBlock, PerBlock};

/// A price in USD and cents per BTC. The field types preserve each family's source, sampling and
/// rounding policy.
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

impl Price<LazyPerBlock<Cents>> {
    pub(crate) fn from_lazy_cents_source(
        name: &str,
        version: Version,
        source: &LazyPerBlock<Cents>,
    ) -> Self {
        let cents =
            LazyPerBlock::from_lazy::<Ident, Cents>(&format!("{name}_cents"), version, source);
        let usd = LazyPerBlock::from_lazy::<Convert, Cents>(name, version, &cents);
        Self { usd, cents }
    }

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

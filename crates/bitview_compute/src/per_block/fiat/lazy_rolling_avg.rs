use bitview_traversable::Traversable;
use brk_types::{Dollars, Height, StoredF32, Version};
use vecdb::{DeltaAvg, LazyDeltaVec, LazyVec, ReadOnlyClone, ReadableCloneableVec};

use crate::{
    AvgCentsToUsd, CachedWindowStartVec, DerivedResolutions, FiatType, LazyPerBlock,
    LazyRollingAvgFromHeight, Resolutions,
};

#[derive(Clone, Traversable)]
pub struct LazyRollingAvgFiatFromHeight<C: FiatType> {
    /// Reported in US dollars.
    pub usd: LazyPerBlock<Dollars, StoredF32>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: LazyRollingAvgFromHeight<C>,
}

impl<C: FiatType> LazyRollingAvgFiatFromHeight<C> {
    pub fn new(
        name: &str,
        version: Version,
        cumulative: &(impl ReadableCloneableVec<Height, C> + 'static),
        cached_start: &CachedWindowStartVec,
        indexes: &crate::IndexSources,
    ) -> Self {
        let cached = cached_start.read_only_clone();
        let average = LazyDeltaVec::<Height, C, StoredF32, DeltaAvg>::new(
            &format!("{name}_cents"),
            version,
            cumulative.read_only_boxed_clone(),
            cached.version(),
            move || cached.snapshot(),
        );
        let resolutions = Resolutions::from_height_source(
            &format!("{name}_cents"),
            average.clone(),
            version,
            indexes,
        );
        let cents = LazyRollingAvgFromHeight {
            height: average,
            resolutions: Box::new(resolutions),
        };
        let usd = LazyPerBlock {
            height: LazyVec::transformed::<AvgCentsToUsd>(
                name,
                version,
                cents.height.read_only_boxed_clone(),
            ),
            resolutions: Box::new(DerivedResolutions::from_derived_computed::<AvgCentsToUsd>(
                name,
                version,
                &cents.resolutions,
            )),
        };

        Self { usd, cents }
    }
}

use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{DeltaSub, LazyDeltaVec, LazyVec, ReadOnlyClone, ReadableCloneableVec};

use crate::{
    CACHE_BUDGET, CachedWindowStartVec, DerivedResolutions, FiatType, LazyPerBlock,
    LazyRollingSumFromHeight, Resolutions, Windows,
};

use super::LazyRollingSumFiatFromHeight;

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyRollingSumsFiatFromHeight<C: FiatType>(
    /// Total of the per-block values over the trailing window ending at the
    /// represented block. At time-period indexes, the value is taken at the
    /// period's final block.
    pub Windows<LazyRollingSumFiatFromHeight<C>>,
);

impl<C: FiatType> LazyRollingSumsFiatFromHeight<C> {
    pub fn new(
        name: &str,
        version: Version,
        cumulative_cents: &(impl ReadableCloneableVec<Height, C> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let cumulative_cents = cumulative_cents.read_only_boxed_clone();

        Self(cached_starts.map_with_suffix(|suffix, cached_start| {
            let name = format!("{name}_{suffix}");
            let cached = cached_start.read_only_clone();
            let starts_version = cached.version();

            let cents_name = format!("{name}_cents");
            let height = LazyDeltaVec::<Height, C, C, DeltaSub>::new(
                &cents_name,
                version,
                cumulative_cents.clone(),
                starts_version,
                move || cached.snapshot(),
            );
            let source = CACHE_BUDGET.wrap(height.clone());
            let resolutions =
                Resolutions::from_height_source(&cents_name, source, version, indexes);
            let cents = LazyRollingSumFromHeight {
                height,
                resolutions: Box::new(resolutions),
            };

            let usd = LazyPerBlock {
                height: LazyVec::transformed::<C::ToDollars>(
                    &name,
                    version,
                    cents.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<C::ToDollars>(
                    &name,
                    version,
                    &cents.resolutions,
                )),
            };

            LazyRollingSumFiatFromHeight { usd, cents }
        }))
    }
}

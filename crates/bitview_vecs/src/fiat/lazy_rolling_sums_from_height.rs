use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Convert, ReadableCloneableVec};

use crate::{
    FiatType, IndexSources, LazyPerBlock, LazyRollingSumFiatFromHeight, PRICED_VERSION,
    PricedRollingSumFromHeight, shapes::Windows,
};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyRollingSumsFiatFromHeight<C: FiatType>(
    /// Total of the per-block values over the trailing window ending at the
    /// represented block. At time-period indexes, the value is taken at the
    /// period's final block. In dollars, undefined (null) until the window lies
    /// at or after the first exchange price (block 74,641): blocks before it are
    /// valued at zero.
    pub Windows<LazyRollingSumFiatFromHeight<C>>,
);

impl<C: FiatType> LazyRollingSumsFiatFromHeight<C> {
    pub(crate) fn new(
        name: &str,
        version: Version,
        cumulative_cents: &impl ReadableCloneableVec<Height, C>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        let version = version + PRICED_VERSION;
        Self(window_starts.map_with_suffix(|suffix, window_start| {
            let name = format!("{name}_{suffix}");
            let cents = PricedRollingSumFromHeight::from_source(
                &format!("{name}_cents"),
                version,
                cumulative_cents,
                *window_start,
                indexes,
            );

            let usd = LazyPerBlock::from_resolutions::<Convert>(&name, version, &cents.resolutions);

            LazyRollingSumFiatFromHeight { usd, cents }
        }))
    }
}

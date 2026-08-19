use bitview_traversable::Traversable;
use brk_types::{Dollars, Height, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

use crate::{FiatType, LazyPerBlock, LazyPreviousDeltaVec};

/// Per-block fiat data derived from stored cumulative cents.
#[derive(Clone, Traversable)]
pub struct LazyFiatBlock<C: FiatType> {
    /// Reported in US dollars.
    pub usd: LazyVec<Height, Dollars, Height, C>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: LazyPreviousDeltaVec<Height, C>,
}

impl<C: FiatType> LazyFiatBlock<C> {
    pub fn from_cumulative_source(
        name: &str,
        version: Version,
        cumulative: &LazyPerBlock<C>,
    ) -> Self {
        let cents = LazyPreviousDeltaVec::new(
            &format!("{name}_cents"),
            version,
            cumulative.height.read_only_boxed_clone(),
        );
        let usd =
            LazyVec::transformed::<C::ToDollars>(name, version, cents.read_only_boxed_clone());
        Self { usd, cents }
    }
}

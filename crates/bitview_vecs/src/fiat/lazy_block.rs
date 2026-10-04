use bitview_transforms::Convert;
use brk_types::{Dollars, Height, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

use crate::{Fiat, FiatType, LazyPerBlock, LazyPreviousDeltaVec};

/// Per-block fiat data derived from stored cumulative cents.
pub type LazyFiatBlock<C> =
    Fiat<LazyPreviousDeltaVec<Height, C>, LazyVec<Height, Dollars, Height, C>>;

impl<C: FiatType> LazyFiatBlock<C> {
    pub(crate) fn from_cumulative_source(
        name: &str,
        version: Version,
        cumulative: &LazyPerBlock<C>,
    ) -> Self {
        let cents =
            LazyPreviousDeltaVec::new(&format!("{name}_cents"), version, &cumulative.height);
        let usd = LazyVec::transformed::<Convert>(name, version, cents.read_only_boxed_clone());
        Self { usd, cents }
    }
}

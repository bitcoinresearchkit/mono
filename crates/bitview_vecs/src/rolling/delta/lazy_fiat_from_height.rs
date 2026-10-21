use brk_types::Dollars;

use crate::{Fiat, LazyPerBlock, PricedDeltaFromHeight};

/// A dollar level's change over a window, in cents and dollars: NaN until it and the value before
/// it are priced.
pub type LazyDeltaFiatFromHeight<S, C> =
    Fiat<PricedDeltaFromHeight<S, C>, LazyPerBlock<Dollars, C>>;

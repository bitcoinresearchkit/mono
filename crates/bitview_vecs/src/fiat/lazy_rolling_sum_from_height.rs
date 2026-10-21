use brk_types::Dollars;

use crate::{Fiat, LazyPerBlock, PricedRollingSumFromHeight};

/// A dollar rolling sum, in cents and dollars: NaN until its window is wholly priced.
pub type LazyRollingSumFiatFromHeight<C> =
    Fiat<PricedRollingSumFromHeight<C>, LazyPerBlock<Dollars, C>>;

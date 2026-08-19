use bitview_traversable::Traversable;
use brk_types::Dollars;
use vecdb::{DeltaChange, VecValue};

use crate::{FiatType, LazyPerBlock};

use super::LazyDeltaFromHeight;

#[derive(Clone, Traversable)]
pub struct LazyDeltaFiatFromHeight<S, C>
where
    S: VecValue,
    C: FiatType,
{
    /// Reported in US dollars.
    pub usd: LazyPerBlock<Dollars, C>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: LazyDeltaFromHeight<S, C, DeltaChange>,
}

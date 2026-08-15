use brk_traversable::Traversable;
use brk_types::Dollars;
use vecdb::{DeltaChange, VecValue};

use crate::internal::{FiatType, LazyPerBlock};

use super::LazyDeltaFromHeight;

#[derive(Clone, Traversable)]
pub struct LazyDeltaFiatFromHeight<S, C>
where
    S: VecValue,
    C: FiatType,
{
    pub usd: LazyPerBlock<Dollars, C>,
    pub cents: LazyDeltaFromHeight<S, C, DeltaChange>,
}

use brk_traversable::Traversable;
use brk_types::Bitcoin;
use vecdb::{DeltaChange, VecValue};

use crate::internal::{AmountType, LazyPerBlock};

use super::LazyDeltaFromHeight;

#[derive(Clone, Traversable)]
pub struct LazyDeltaAmountFromHeight<S, C>
where
    S: VecValue,
    C: AmountType,
{
    pub btc: LazyPerBlock<Bitcoin, C>,
    pub sats: LazyDeltaFromHeight<S, C, DeltaChange>,
}

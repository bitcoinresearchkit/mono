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
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: LazyPerBlock<Bitcoin, C>,
    /// Reported in satoshis.
    pub sats: LazyDeltaFromHeight<S, C, DeltaChange>,
}

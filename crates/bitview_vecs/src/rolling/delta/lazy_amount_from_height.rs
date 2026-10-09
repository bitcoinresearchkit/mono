use bitview_traversable::Traversable;
use brk_types::Bitcoin;
use vecdb::{DeltaChange, VecValue};

use crate::{AmountType, LazyPerBlock};

use crate::LazyDeltaFromHeight;

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyDeltaAmountFromHeight<S, C>
where
    S: VecValue,
    C: AmountType,
{
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub(crate) btc: LazyPerBlock<Bitcoin, C>,
    /// Reported in satoshis.
    #[traversable(hidden)]
    pub(crate) sats: LazyDeltaFromHeight<S, C, DeltaChange>,
}

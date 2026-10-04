use bitview_compute::Quantity;
use vecdb::DeltaAvg;

use crate::LazyDeltaFromHeight;

/// A lazy rolling average from height and its resolution views, in the quantity's mean type.
pub type LazyRollingAvgFromHeight<T> = LazyDeltaFromHeight<T, <T as Quantity>::Fract, DeltaAvg>;

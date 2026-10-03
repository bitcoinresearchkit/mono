use bitview_primitives::StoredF32;
use vecdb::DeltaAvg;

use crate::LazyDeltaFromHeight;

/// A lazy rolling average from height and its resolution views, reported as StoredF32.
pub type LazyRollingAvgFromHeight<T> = LazyDeltaFromHeight<T, StoredF32, DeltaAvg>;

use bitview_primitives::StoredU64;
use brk_types::{Cents, Height};
use vecdb::LazyDeltaVec;

use super::sma_average::SmaAverage;

/// Price SMA uses the same bounded current/lookback reads as other rolling deltas.
pub type LazySmaVec = LazyDeltaVec<Height, StoredU64, Cents, SmaAverage>;

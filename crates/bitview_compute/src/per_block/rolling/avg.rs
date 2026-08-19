use bitview_traversable::Traversable;
use brk_types::{Height, StoredF32};
use schemars::JsonSchema;
use vecdb::{DeltaAvg, LazyDeltaVec};

use crate::{NumericValue, Resolutions};

/// A single lazy rolling-average slot from height: the lazy delta vec + its resolution views.
/// Output is always StoredF32 regardless of input type T.
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyRollingAvgFromHeight<T>
where
    T: NumericValue + JsonSchema,
{
    pub height: LazyDeltaVec<Height, T, StoredF32, DeltaAvg>,
    #[traversable(flatten)]
    pub resolutions: Box<Resolutions<StoredF32>>,
}

use bitview_traversable::Traversable;
use brk_types::Height;
use schemars::JsonSchema;
use vecdb::{DeltaSub, LazyDeltaVec};

use crate::{NumericValue, Resolutions};

/// A single lazy rolling-sum slot from height: the lazy delta vec + its resolution views.
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyRollingSumFromHeight<T>
where
    T: NumericValue + JsonSchema,
{
    pub height: LazyDeltaVec<Height, T, T, DeltaSub>,
    #[traversable(flatten)]
    pub resolutions: Box<Resolutions<T>>,
}

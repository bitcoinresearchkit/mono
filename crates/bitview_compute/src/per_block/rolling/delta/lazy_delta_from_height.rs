use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{DeltaOp, LazyDeltaVec, VecValue};

use crate::{CACHE_BUDGET, NumericValue, Resolutions};

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyDeltaFromHeight<S, T, Op: 'static>
where
    S: VecValue,
    T: NumericValue + JsonSchema,
{
    pub height: LazyDeltaVec<Height, S, T, Op>,
    #[traversable(flatten)]
    pub resolutions: Box<Resolutions<T>>,
}

impl<S, T, Op> LazyDeltaFromHeight<S, T, Op>
where
    S: VecValue,
    T: NumericValue + JsonSchema,
    Op: DeltaOp<S, T>,
{
    pub fn new(
        name: &str,
        version: Version,
        height: LazyDeltaVec<Height, S, T, Op>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let source = CACHE_BUDGET.wrap(height.clone());
        let resolutions = Resolutions::from_height_source(name, source, version, indexes);

        Self {
            height,
            resolutions: Box::new(resolutions),
        }
    }
}

use bitview_compute::NumericValue;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{DeltaOp, LazyDeltaVec, ReadableCloneableVec, VecValue};

use crate::{IndexSources, Resolutions};

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyDeltaFromHeight<S, T, Op: 'static>
where
    S: VecValue,
    T: NumericValue + JsonSchema,
{
    pub height: LazyDeltaVec<Height, S, T, Op>,
    #[traversable(flatten)]
    pub(crate) resolutions: Box<Resolutions<T>>,
}

impl<S, T, Op> LazyDeltaFromHeight<S, T, Op>
where
    S: VecValue + Default,
    T: NumericValue + JsonSchema,
    Op: DeltaOp<S, T>,
{
    pub(crate) fn from_source(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        window_start: &impl ReadableCloneableVec<Height, Height>,
        indexes: &IndexSources,
    ) -> Self {
        let window_start = window_start.read_only_boxed_clone();
        let height = LazyDeltaVec::new(name, version, source.read_only_boxed_clone(), window_start);
        let resolutions = Resolutions::from_source(name, &height, version, indexes);
        Self {
            height,
            resolutions: Box::new(resolutions),
        }
    }
}

use bitview_traversable::Traversable;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{LazyVec, ReadableBoxedVec, UnaryTransform, VecIndex, VecValue};

use brk_types::Version;

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyTransformLast<I, T, S1T = T>(pub LazyVec<I, T, I, S1T>)
where
    I: VecIndex,
    T: VecValue + PartialOrd + JsonSchema,
    S1T: VecValue;

impl<I, T, S1T> LazyTransformLast<I, T, S1T>
where
    I: VecIndex,
    T: VecValue + PartialOrd + JsonSchema + 'static,
    S1T: VecValue + JsonSchema,
{
    pub fn from_boxed<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<I, S1T>,
    ) -> Self {
        Self(LazyVec::transformed::<F>(name, version, source))
    }
}

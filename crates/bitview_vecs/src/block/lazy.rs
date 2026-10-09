use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    LazyVec, ReadOnlyClone, ReadableBoxedVec, ReadableCloneableVec, UnaryTransform, VecValue,
};

use crate::{DerivedResolutions, IndexSources, Resolutions};
use bitview_compute::ComputedVecValue;

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(merge)]
pub struct LazyPerBlock<T, S1T = T>
where
    T: VecValue + PartialOrd + JsonSchema,
    S1T: VecValue,
{
    pub height: LazyVec<Height, T, Height, S1T>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    resolutions: Box<DerivedResolutions<T, S1T>>,
}

impl<T, S1T> LazyPerBlock<T, S1T>
where
    T: VecValue + PartialOrd + JsonSchema + 'static,
    S1T: VecValue + PartialOrd + JsonSchema,
{
    /// Reuse the resolutions' source and cache for the height view as well.
    pub fn from_resolutions<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        resolutions: &Resolutions<S1T>,
    ) -> Self {
        Self {
            height: LazyVec::transformed::<F>(
                name,
                version,
                resolutions.height_source().read_only_boxed_clone(),
            ),
            resolutions: Box::new(DerivedResolutions::from_derived_computed::<F>(
                name,
                version,
                resolutions,
            )),
        }
    }

    /// Build uncached views. Cache ownership belongs to the source, not its views.
    pub fn from_height_source<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        height_source: &(impl ReadableCloneableVec<Height, S1T> + ?Sized),
        indexes: &IndexSources,
    ) -> Self {
        let resolutions = Resolutions::from_source(name, height_source, version, indexes);
        Self::from_resolutions::<F>(name, version, &resolutions)
    }

    /// Create by unary-transforming a LazyPerBlock source (chaining lazy vecs).
    pub fn from_lazy<F, S2T>(name: &str, version: Version, source: &LazyPerBlock<S1T, S2T>) -> Self
    where
        F: UnaryTransform<S1T, T>,
        S2T: ComputedVecValue + JsonSchema,
    {
        Self {
            height: LazyVec::transformed::<F>(
                name,
                version,
                ReadableBoxedVec::new(source.height.clone()),
            ),
            resolutions: Box::new(DerivedResolutions::from_lazy::<F, S2T>(
                name,
                version,
                &source.resolutions,
            )),
        }
    }
}

impl<T, S1T> ReadOnlyClone for LazyPerBlock<T, S1T>
where
    T: VecValue + PartialOrd + JsonSchema,
    S1T: VecValue,
{
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

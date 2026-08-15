use brk_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    LazyVec, PcoVecValue, ReadOnlyClone, ReadableBoxedVec, ReadableCloneableVec, UnaryTransform,
    VecValue,
};

use crate::{
    indexes,
    internal::{
        CachedPerBlock, ComputedVecValue, DerivedResolutions, NumericValue, PerBlock, Resolutions,
    },
};

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
    pub resolutions: Box<DerivedResolutions<T, S1T>>,
}

impl<T, S1T> LazyPerBlock<T, S1T>
where
    T: VecValue + PartialOrd + JsonSchema + 'static,
    S1T: VecValue + PartialOrd + JsonSchema,
{
    pub(crate) fn from_resolutions<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        height_source: ReadableBoxedVec<Height, S1T>,
        resolutions: &Resolutions<S1T>,
    ) -> Self {
        Self {
            height: LazyVec::transformed::<F>(name, version, height_source),
            resolutions: Box::new(DerivedResolutions::from_derived_computed::<F>(
                name,
                version,
                resolutions,
            )),
        }
    }

    pub(crate) fn from_computed<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        height_source: ReadableBoxedVec<Height, S1T>,
        source: &PerBlock<S1T>,
    ) -> Self
    where
        S1T: PcoVecValue,
    {
        Self::from_resolutions::<F>(name, version, height_source, &source.resolutions)
    }

    pub(crate) fn from_cached_computed<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        height_source: ReadableBoxedVec<Height, S1T>,
        source: &CachedPerBlock<S1T>,
    ) -> Self
    where
        S1T: NumericValue,
    {
        Self::from_resolutions::<F>(name, version, height_source, &source.resolutions)
    }

    pub(crate) fn from_height_source<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        height_source: impl ReadableCloneableVec<Height, S1T> + 'static,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        S1T: NumericValue,
    {
        Self::from_boxed_height_source::<F>(name, version, Box::new(height_source), indexes)
    }

    pub(crate) fn from_boxed_height_source<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        height_source: ReadableBoxedVec<Height, S1T>,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        S1T: NumericValue,
    {
        Self {
            height: LazyVec::transformed::<F>(name, version, height_source.clone()),
            resolutions: Box::new(DerivedResolutions::from_height_source::<F>(
                name,
                version,
                height_source,
                indexes,
            )),
        }
    }

    /// Create by unary-transforming a LazyPerBlock source (chaining lazy vecs).
    pub(crate) fn from_lazy<F, S2T>(
        name: &str,
        version: Version,
        source: &LazyPerBlock<S1T, S2T>,
    ) -> Self
    where
        F: UnaryTransform<S1T, T>,
        S2T: ComputedVecValue + JsonSchema,
    {
        Self {
            height: LazyVec::transformed::<F>(name, version, source.height.read_only_boxed_clone()),
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

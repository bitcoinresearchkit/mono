use bitview_traversable::Traversable;
use brk_types::{Height, PartsPerMillionSigned64, StoredF32, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    BinaryTransform, CachedBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec, UnaryTransform,
    VecValue,
};

use crate::{
    CACHE_BUDGET, Cagr, FixedRatio, Identity, LazyIndexedVec, LazyLookbackVec, LazyPerBlock,
    NumericValue, Percent,
};

/// Fully lazy variant of `PercentPerBlock` — no stored vecs.
///
/// PPM values are lazily derived from one source, and ratio/percent float views
/// are chained from them.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyPercentPerBlock<B: FixedRatio>(
    pub Percent<LazyPerBlock<B, B>, LazyPerBlock<StoredF32, B>>,
);

impl<B: FixedRatio> LazyPercentPerBlock<B> {
    pub fn from_cached_ratio<S, D, F>(
        name: &str,
        version: Version,
        numerator: &(impl ReadableCloneableVec<Height, S> + 'static),
        denominator: CachedBoxedVec<Height, D>,
        indexes: &crate::IndexSources,
    ) -> Self
    where
        S: NumericValue,
        D: NumericValue,
        F: BinaryTransform<S, D, B> + Send + Sync + 'static,
    {
        let source = LazyIndexedVec::new(
            &format!("{name}_{}_source", B::SUFFIX),
            version,
            numerator.read_only_boxed_clone(),
            denominator,
            |_, numerator, denominator| F::apply(numerator, denominator),
        );
        let source = CACHE_BUDGET.wrap(source);
        Self::from_height_source(name, version, source, indexes)
    }

    pub fn from_ratio_with_cached_numerator<S, D, F>(
        name: &str,
        version: Version,
        numerator: CachedBoxedVec<Height, S>,
        denominator: &(impl ReadableCloneableVec<Height, D> + 'static),
        indexes: &crate::IndexSources,
    ) -> Self
    where
        S: NumericValue,
        D: NumericValue,
        F: BinaryTransform<S, D, B> + Send + Sync + 'static,
    {
        let source = LazyIndexedVec::new(
            &format!("{name}_{}_source", B::SUFFIX),
            version,
            denominator.read_only_boxed_clone(),
            numerator,
            |_, denominator, numerator| F::apply(numerator, denominator),
        );
        let source = CACHE_BUDGET.wrap(source);
        Self::from_height_source(name, version, source, indexes)
    }

    pub fn from_height_source<V>(
        name: &str,
        version: Version,
        source: V,
        indexes: &crate::IndexSources,
    ) -> Self
    where
        V: TypedVec<I = Height, T = B> + ReadableVec<Height, B> + Clone + 'static,
    {
        let ppm_name = format!("{name}_{}", B::SUFFIX);
        let ppm =
            LazyPerBlock::from_height_source::<Identity<B>>(&ppm_name, version, source, indexes);
        Self::from_ppm(name, version, ppm)
    }

    /// Create from two values a fixed distance apart in one height source.
    pub fn from_lookback_source<S>(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        lookback: usize,
        compute: fn(S, Option<S>) -> B,
        indexes: &crate::IndexSources,
    ) -> Self
    where
        S: VecValue,
    {
        let ppm_name = format!("{name}_{}", B::SUFFIX);
        let source = LazyLookbackVec::new(
            &format!("{ppm_name}_source"),
            version,
            source.read_only_boxed_clone(),
            lookback,
            compute,
        );
        let source = CACHE_BUDGET.wrap(source);
        let ppm =
            LazyPerBlock::from_height_source::<Identity<B>>(&ppm_name, version, source, indexes);

        Self::from_ppm(name, version, ppm)
    }

    pub fn from_lazy_percent<F: UnaryTransform<B, B>>(
        name: &str,
        version: Version,
        source: &Self,
    ) -> Self {
        let ppm =
            LazyPerBlock::from_lazy::<F, B>(&format!("{name}_{}", B::SUFFIX), version, &source.ppm);
        Self::from_ppm(name, version, ppm)
    }

    fn from_ppm(name: &str, version: Version, ppm: LazyPerBlock<B, B>) -> Self {
        let ratio =
            LazyPerBlock::from_lazy::<B::ToRatio, B>(&format!("{name}_ratio"), version, &ppm);
        let percent = LazyPerBlock::from_lazy::<B::ToPercent, B>(name, version, &ppm);
        Self(Percent {
            ppm,
            ratio,
            percent,
        })
    }
}

impl LazyPercentPerBlock<PartsPerMillionSigned64> {
    pub fn from_lazy_cagr(name: &str, version: Version, years: u8, source: &Self) -> Self {
        match years {
            2 => Self::from_lazy_percent::<Cagr<2>>(name, version, source),
            3 => Self::from_lazy_percent::<Cagr<3>>(name, version, source),
            4 => Self::from_lazy_percent::<Cagr<4>>(name, version, source),
            5 => Self::from_lazy_percent::<Cagr<5>>(name, version, source),
            6 => Self::from_lazy_percent::<Cagr<6>>(name, version, source),
            8 => Self::from_lazy_percent::<Cagr<8>>(name, version, source),
            10 => Self::from_lazy_percent::<Cagr<10>>(name, version, source),
            _ => unreachable!("unsupported DCA CAGR period: {years} years"),
        }
    }
}

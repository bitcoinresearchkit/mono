use std::{marker::PhantomData, sync::Arc};

use rangeindex::SharedRangeMap;

pub mod any_vec;
pub mod clone;
pub mod fold;
pub mod readable;
pub mod sparse;
mod traversable;
pub mod typed;

pub use fold::*;
use sparse::Sparse;

use vecdb::{AnyVec, ReadableBoxedVec, VecIndex, VecValue, Version};

use crate::RangeMapVec;

/// Lazy aggregation vector that maps coarser output indices to ranges in a finer source.
///
/// Values are computed on demand from a readable source and resident boundaries.
pub struct LazyAggVec<I, O, S1I, S1T = O, Strat = Sparse>
where
    I: VecIndex,
    O: VecValue,
    S1I: VecIndex,
    S1T: VecValue,
{
    name: Arc<str>,
    version: Version,
    source: ReadableBoxedVec<S1I, S1T>,
    mapping: SharedRangeMap<S1I, I>,
    #[allow(clippy::type_complexity)]
    _phantom: PhantomData<fn() -> (I, O, Strat)>,
}

impl<I, O, S1I, S1T, Strat> LazyAggVec<I, O, S1I, S1T, Strat>
where
    I: VecIndex,
    O: VecValue,
    S1I: VecIndex,
    S1T: VecValue,
{
    /// The shared source read by this aggregation view.
    pub(crate) fn source(&self) -> &ReadableBoxedVec<S1I, S1T> {
        &self.source
    }

    /// The view's version is `version` plus the boundary mapping's.
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<S1I, S1T>,
        mapping: &RangeMapVec<I, S1I>,
    ) -> Self
    where
        S1I: VecValue,
    {
        Self {
            name: Arc::from(name),
            version: version + mapping.version(),
            source,
            mapping: mapping.mapping().clone(),
            _phantom: PhantomData,
        }
    }
}

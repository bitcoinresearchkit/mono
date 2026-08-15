use brk_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::ReadableBoxedVec;

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, FixedRatio, Identity, LazyPerBlock, LazyRollingDeltasFromHeight,
        NumericValue, Windows,
    },
};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyPerBlockWithDeltas<S, C, B>
where
    S: NumericValue + JsonSchema + Into<f64>,
    C: NumericValue + JsonSchema + From<f64>,
    B: FixedRatio + From<f64>,
{
    #[deref]
    #[deref_mut]
    pub base: LazyPerBlock<S>,
    pub delta: LazyRollingDeltasFromHeight<S, C, B>,
}

impl<S, C, B> LazyPerBlockWithDeltas<S, C, B>
where
    S: NumericValue + JsonSchema + Into<f64>,
    C: NumericValue + JsonSchema + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub(crate) fn from_boxed_height_source(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, S>,
        delta_version_offset: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let base =
            LazyPerBlock::from_boxed_height_source::<Identity<S>>(name, version, source, indexes);
        let delta = LazyRollingDeltasFromHeight::new(
            &format!("{name}_delta"),
            version + delta_version_offset,
            &base.height,
            cached_starts,
            indexes,
        );
        Self { base, delta }
    }
}

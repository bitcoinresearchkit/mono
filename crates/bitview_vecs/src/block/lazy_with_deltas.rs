use bitview_collections::Windows;
use bitview_compute::{FixedRatio, NumericValue};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Ident, ReadableCloneableVec};

use crate::{IndexSources, LazyPerBlock, LazyRollingDeltasFromHeight};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyPerBlockWithDeltas<S, C, B>
where
    S: NumericValue + JsonSchema + Into<f64>,
    C: NumericValue + JsonSchema + From<f64>,
    B: FixedRatio + From<f64>,
{
    #[deref]
    #[deref_mut]
    block: LazyPerBlock<S>,
    delta: LazyRollingDeltasFromHeight<S, C, B>,
}

impl<S, C, B> LazyPerBlockWithDeltas<S, C, B>
where
    S: NumericValue + JsonSchema + Into<f64>,
    C: NumericValue + JsonSchema + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn from_height_source(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + ?Sized),
        delta_version_offset: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Self {
        let block = LazyPerBlock::from_height_source::<Ident>(name, version, source, indexes);
        let delta = LazyRollingDeltasFromHeight::new(
            &format!("{name}_delta"),
            version + delta_version_offset,
            &block.height,
            window_starts,
            indexes,
        );
        Self { block, delta }
    }
}

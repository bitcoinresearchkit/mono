use bitview_primitives::{BasisPoints32, StoredF32};
use bitview_transforms::FixedToRatio;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{Ident, ReadableCloneableVec};

use crate::{IndexSources, LazyPerBlock};

/// Basis-point and decimal views derived lazily from one source.
#[derive(Clone, Traversable)]
pub struct LazyBasisPointsPerBlock {
    /// Unitless ratio in basis points; 10,000 represents 1.0. Floored to whole
    /// basis points, with u32::MAX reserved for undefined values.
    bps: LazyPerBlock<BasisPoints32>,
    /// Unitless decimal ratio derived as basis points divided by 10,000.
    ratio: LazyPerBlock<StoredF32, BasisPoints32>,
}

impl LazyBasisPointsPerBlock {
    pub fn from_height_source<V>(
        name: &str,
        version: Version,
        source: &V,
        indexes: &IndexSources,
    ) -> Self
    where
        V: ReadableCloneableVec<Height, BasisPoints32> + ?Sized,
    {
        let bps = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_bps"),
            version,
            source,
            indexes,
        );
        let ratio = LazyPerBlock::from_lazy::<FixedToRatio, BasisPoints32>(name, version, &bps);
        Self { bps, ratio }
    }
}

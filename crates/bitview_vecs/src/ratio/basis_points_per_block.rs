use bitview_transforms::FixedToRatio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{BasisPoints32, StoredF32, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::{IndexSources, LazyPerBlock, PerBlock};

/// Basis-point storage with a lazy decimal view.
#[derive(Traversable)]
pub struct BasisPointsPerBlock<M: StorageMode = Rw> {
    /// Unitless ratio in basis points; 10,000 represents 1.0. Floored to whole
    /// basis points, with u32::MAX reserved for undefined values.
    pub bps: PerBlock<BasisPoints32, M>,
    /// Unitless decimal ratio derived as basis points divided by 10,000.
    ratio: LazyPerBlock<StoredF32, BasisPoints32>,
}

impl BasisPointsPerBlock {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let bps = PerBlock::import(db, &format!("{name}_bps"), version, indexes)?;
        let ratio = LazyPerBlock::from_resolutions::<FixedToRatio>(name, version, &bps);
        Ok(Self { bps, ratio })
    }
}

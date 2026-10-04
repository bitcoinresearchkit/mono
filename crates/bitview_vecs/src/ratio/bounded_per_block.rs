use bitview_primitives::{BoundedRatio, Ratio64};
use bitview_transforms::BoundedToRatio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use crate::{IndexSources, LazyPerBlock, PerBlock};

/// Bounded fixed-point storage with a lazy decimal view.
#[derive(Traversable)]
pub struct BoundedRatioPerBlock<M: StorageMode = Rw> {
    /// Encoded share in [0, 4,294,967,294]; u32::MAX means undefined.
    pub bounded: PerBlock<BoundedRatio, M>,
    /// Unitless decimal share derived from the bounded values.
    pub ratio: LazyPerBlock<Ratio64, BoundedRatio>,
}

impl BoundedRatioPerBlock {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let bounded = PerBlock::import(db, &format!("{name}_bounded"), version, indexes)?;
        let ratio = LazyPerBlock::from_resolutions::<BoundedToRatio>(name, version, &bounded);
        Ok(Self { bounded, ratio })
    }
}

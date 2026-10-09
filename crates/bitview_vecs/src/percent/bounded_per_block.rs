use bitview_primitives::{BoundedRatio, Percent};
use bitview_transforms::FixedToPercent;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use crate::{IndexSources, LazyPerBlock, PerBlock};

/// Bounded fixed-point storage with a lazy percent view.
#[derive(Traversable)]
#[traversable(merge)]
pub struct BoundedPercentPerBlock<M: StorageMode = Rw> {
    /// Encoded share in [0, 4,294,967,294]; u32::MAX means undefined.
    #[traversable(hidden)]
    pub fixed: PerBlock<BoundedRatio, M>,
    /// As a percentage.
    percent: LazyPerBlock<Percent, BoundedRatio>,
}

impl BoundedPercentPerBlock {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let fixed = PerBlock::import(db, &format!("{name}_bounded"), version, indexes)?;
        let percent = LazyPerBlock::from_resolutions::<FixedToPercent>(name, version, &fixed);
        Ok(Self { fixed, percent })
    }
}

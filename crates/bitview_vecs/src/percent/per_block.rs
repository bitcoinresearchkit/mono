use bitview_compute::FixedRatio;
use bitview_primitives::Percent;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use crate::{IndexSources, LazyPerBlock, PerBlock, ops::FixedToPercent, shapes::PercentViews};

/// Fixed-point storage and its one public view, as a percentage.
#[derive(Deref, DerefMut, Traversable)]
pub struct PercentPerBlock<B: FixedRatio, M: StorageMode = Rw>(
    pub PercentViews<PerBlock<B, M>, LazyPerBlock<Percent, B>>,
);

impl<B: FixedRatio> PercentPerBlock<B> {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let fixed = PerBlock::import(db, &format!("{name}_{}", B::SUFFIX), version, indexes)?;

        let percent = LazyPerBlock::from_resolutions::<FixedToPercent>(name, version, &fixed);

        Ok(Self(PercentViews { fixed, percent }))
    }
}

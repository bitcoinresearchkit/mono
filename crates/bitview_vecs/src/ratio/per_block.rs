use bitview_compute::FixedRatio;
use bitview_primitives::Ratio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use crate::{IndexSources, LazyPerBlock, PerBlock, RatioValue, ops::FixedToRatio};

/// Fixed-point storage and its one public view, as a ratio; `O` the public value, a plain [`Ratio`]
/// or a [`bitview_primitives::Multiple`].
#[derive(Traversable)]
#[traversable(merge)]
pub struct RatioPerBlock<R: FixedRatio, M: StorageMode = Rw, O: RatioValue = Ratio> {
    /// Fixed-point ratio in parts per million; 1,000,000 represents 1.0.
    #[traversable(hidden)]
    pub fixed: PerBlock<R, M>,
    /// As a ratio.
    pub ratio: LazyPerBlock<O, R>,
}

const VERSION: Version = Version::new(3);

impl<R: FixedRatio, O: RatioValue> RatioPerBlock<R, Rw, O> {
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let v = version + VERSION;

        let fixed = PerBlock::import(db, &format!("{name}_{}", R::SUFFIX), v, indexes)?;

        let ratio = LazyPerBlock::from_resolutions::<FixedToRatio>(name, v, &fixed);

        Ok(Self { fixed, ratio })
    }
}

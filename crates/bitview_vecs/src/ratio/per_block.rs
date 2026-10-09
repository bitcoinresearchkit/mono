use bitview_compute::FixedRatio;
use bitview_primitives::Ratio;
use bitview_transforms::FixedToRatio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{BinaryTransform, Database, ReadableVec, Rw, StorageMode, VecValue};

use crate::{IndexSources, LazyPerBlock, PerBlock};

/// Fixed-point storage and its one public view, as a ratio.
#[derive(Traversable)]
#[traversable(merge)]
pub struct RatioPerBlock<R: FixedRatio, M: StorageMode = Rw> {
    /// Fixed-point storage: parts per million (1,000,000 represents 1.0) or basis points.
    #[traversable(hidden)]
    pub fixed: PerBlock<R, M>,
    /// As a ratio.
    pub ratio: LazyPerBlock<Ratio, R>,
}

const VERSION: Version = Version::new(3);

impl<R: FixedRatio> RatioPerBlock<R> {
    pub fn import(
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

    pub fn compute_binary<S1T, S2T, F>(
        &mut self,
        max_from: Height,
        source1: &impl ReadableVec<Height, S1T>,
        source2: &impl ReadableVec<Height, S2T>,
        exit: &Exit,
    ) -> Result<()>
    where
        S1T: VecValue,
        S2T: VecValue,
        F: BinaryTransform<S1T, S2T, R>,
    {
        self.fixed
            .compute_binary::<S1T, S2T, F>(max_from, source1, source2, exit)
    }
}

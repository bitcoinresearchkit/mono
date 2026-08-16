use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, StoredF32, Version};
use vecdb::{Database, Exit, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use crate::{
    indexes,
    internal::{FixedRatio, LazyPerBlock, PerBlock},
};

#[derive(Traversable)]
pub struct RatioPerBlock<R: FixedRatio, M: StorageMode = Rw> {
    /// Unitless ratio in parts per million; 1,000,000 represents 1.0.
    pub ppm: PerBlock<R, M>,
    /// Unitless decimal ratio derived as parts per million divided by 1,000,000.
    pub ratio: LazyPerBlock<StoredF32, R>,
}

const VERSION: Version = Version::new(3);

impl<R: FixedRatio> RatioPerBlock<R> {
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        Self::forced_import_ppm(db, &format!("{name}_ratio"), version, indexes)
    }

    pub(crate) fn forced_import_ppm(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let v = version + VERSION;

        let ppm = PerBlock::forced_import(db, &format!("{name}_{}", R::SUFFIX), v, indexes)?;

        let ratio = LazyPerBlock::from_computed::<R::ToRatio>(
            name,
            v,
            ppm.height.read_only_boxed_clone(),
            &ppm,
        );

        Ok(Self { ppm, ratio })
    }

    pub(crate) fn compute_ratio(
        &mut self,
        starting_lengths: &Lengths,
        close_price: &impl ReadableVec<Height, Cents>,
        series_price: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.ppm.height.compute_transform2(
            starting_lengths.height,
            close_price,
            series_price,
            |(i, close, price, ..)| {
                if price == Cents::ZERO {
                    (i, R::from(1.0))
                } else {
                    (i, R::from(f64::from(close) / f64::from(price)))
                }
            },
            exit,
        )?;
        Ok(())
    }
}

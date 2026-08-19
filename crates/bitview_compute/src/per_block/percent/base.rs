use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, StoredF32, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    BinaryTransform, Database, EagerVec, Exit, PcoVec, ReadableCloneableVec, ReadableVec, Rw,
    StorageMode, VecValue,
};

use crate::{FixedRatio, Percent, algo::ComputeDrawdown};

use crate::{LazyPerBlock, PerBlock};

/// Fixed-point storage with lazy ratio and percentage float views.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct PercentPerBlock<B: FixedRatio, M: StorageMode = Rw>(
    pub Percent<PerBlock<B, M>, LazyPerBlock<StoredF32, B>>,
);

impl<B: FixedRatio> PercentPerBlock<B> {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        let ppm = PerBlock::forced_import(db, &format!("{name}_{}", B::SUFFIX), version, indexes)?;
        let ppm_clone = ppm.height.read_only_boxed_clone();

        let ratio = LazyPerBlock::from_computed::<B::ToRatio>(
            &format!("{name}_ratio"),
            version,
            ppm_clone.clone(),
            &ppm,
        );

        let percent = LazyPerBlock::from_computed::<B::ToPercent>(name, version, ppm_clone, &ppm);

        Ok(Self(Percent {
            ppm,
            ratio,
            percent,
        }))
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
        F: BinaryTransform<S1T, S2T, B>,
    {
        self.ppm
            .compute_binary::<S1T, S2T, F>(max_from, source1, source2, exit)
    }

    pub fn compute_drawdown<C, A>(
        &mut self,
        max_from: Height,
        current: &impl ReadableVec<Height, C>,
        ath: &impl ReadableVec<Height, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        C: VecValue,
        A: VecValue,
        f64: From<C> + From<A>,
        EagerVec<PcoVec<Height, B>>: ComputeDrawdown<Height>,
    {
        self.ppm
            .height
            .compute_drawdown(max_from, current, ath, exit)
    }
}

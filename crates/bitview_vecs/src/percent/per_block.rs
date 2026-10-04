use bitview_collections::FixedRatioViews;
use bitview_compute::{ComputeDrawdown, FixedRatio};
use bitview_primitives::{Percent, Ratio};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    BinaryTransform, Budgeted, Database, EagerVec, PcoVec, ReadableVec, Rw, StorageMode, VecValue,
};

use crate::{IndexSources, LazyPerBlock, PerBlock};

/// Fixed-point storage with lazy ratio and percentage float views.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct PercentPerBlock<B: FixedRatio, M: StorageMode = Rw>(
    pub FixedRatioViews<PerBlock<B, M>, LazyPerBlock<Ratio, B>, LazyPerBlock<Percent, B>>,
);

impl<B: FixedRatio> PercentPerBlock<B> {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let ppm = PerBlock::import(db, &format!("{name}_{}", B::SUFFIX), version, indexes)?;

        let ratio =
            LazyPerBlock::from_resolutions::<B::ToRatio>(&format!("{name}_ratio"), version, &ppm);

        let percent = LazyPerBlock::from_resolutions::<B::ToPercent>(name, version, &ppm);

        Ok(Self(FixedRatioViews {
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
        EagerVec<PcoVec<Height, B, Budgeted>>: ComputeDrawdown<Height>,
    {
        self.ppm
            .height
            .compute_drawdown(max_from, current, ath, exit)
    }
}

use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{EmptyAddrData, EmptyAddrIndex, FundedAddrData, FundedAddrIndex};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, LazyVec, Rw, StorageMode};

use super::{
    AddrActivityVecs, AddrCountsVecs, AddrMetricsState, AvgAmountVecs, DeltaVecs, ExposedAddrVecs,
    FundedAddrCountsVecs, NewAddrCountVecs, ReusedAddrVecs, TotalAddrCountVecs,
};

#[derive(Traversable)]
pub struct AddrVecs<M: StorageMode = Rw> {
    pub funded: FundedAddrCountsVecs<M>,
    pub empty: AddrCountsVecs<M>,
    pub activity: AddrActivityVecs<M>,
    pub total: TotalAddrCountVecs<M>,
    pub new: NewAddrCountVecs,
    pub reused: ReusedAddrVecs<M>,
    pub respent: ReusedAddrVecs<M>,
    pub exposed: ExposedAddrVecs<M>,
    pub delta: DeltaVecs,
    pub avg_amount: AvgAmountVecs<M>,
    #[traversable(wrap = "indexes", rename = "funded")]
    pub funded_index: LazyVec<FundedAddrIndex, FundedAddrIndex, FundedAddrIndex, FundedAddrData>,
    #[traversable(wrap = "indexes", rename = "empty")]
    pub empty_index: LazyVec<EmptyAddrIndex, EmptyAddrIndex, EmptyAddrIndex, EmptyAddrData>,
}

impl AddrVecs {
    pub fn reset_height(&mut self) -> Result<()> {
        self.funded.reset_height()?;
        self.empty.reset_height()?;
        self.activity.reset_height()?;
        self.total.reset_height()?;
        self.reused.reset_height()?;
        self.respent.reset_height()?;
        self.exposed.reset_height()?;
        self.avg_amount.reset_height()?;
        Ok(())
    }

    pub fn min_resume_len(&self) -> usize {
        self.funded
            .min_resume_len()
            .min(self.empty.min_resume_len())
            .min(self.activity.min_resume_len())
            .min(self.reused.min_resume_len())
            .min(self.respent.min_resume_len())
            .min(self.exposed.min_resume_len())
    }

    pub fn par_iter_stateful_height_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.funded
            .par_iter_height_mut()
            .chain(self.empty.par_iter_height_mut())
            .chain(self.activity.par_iter_height_mut())
            .chain(self.reused.par_iter_stateful_height_mut())
            .chain(self.respent.par_iter_stateful_height_mut())
            .chain(self.exposed.par_iter_stateful_height_mut())
    }

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.funded
            .par_iter_height_mut()
            .chain(self.empty.par_iter_height_mut())
            .chain(self.activity.par_iter_height_mut())
            .chain(self.total.par_iter_height_mut())
            .chain(self.reused.par_iter_height_mut())
            .chain(self.respent.par_iter_height_mut())
            .chain(self.exposed.par_iter_height_mut())
            .chain(self.avg_amount.par_iter_height_mut())
    }

    #[inline(always)]
    pub fn push_height(&mut self, state: &AddrMetricsState, active_addr_count: u32) {
        self.funded.push_counts(&state.funded);
        self.empty.push_counts(&state.empty);
        self.activity.push_height(&state.activity);
        self.exposed.push_height(&state.exposed);
        self.reused.push_height(&state.reused, active_addr_count);
        self.respent.push_height(&state.respent, active_addr_count);
    }
}

use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{EmptyAddrData, EmptyAddrIndex, FundedAddrData, FundedAddrIndex};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, LazyVec, Rw, StorageMode};

use super::{
    AddrActivityVecs, AddrCountsVecs, AddrMetricsState, AvgAmountVecs, DeltaVecs, ExposedAddrVecs,
    FundedAddrCountsVecs, NewAddrCountVecs, ReusedAddrVecs, TotalAddrCountVecs,
};

#[derive(Traversable)]
pub struct AddrVecs<M: StorageMode = Rw> {
    /// Addresses that currently hold at least one unspent output.
    pub funded: FundedAddrCountsVecs<M>,
    /// Previously seen addresses that currently hold no unspent outputs.
    pub empty: AddrCountsVecs<M>,
    pub activity: AddrActivityVecs<M>,
    /// All previously seen addresses, equal to funded addresses plus empty
    /// addresses.
    pub total: TotalAddrCountVecs<M>,
    /// Addresses first observed in each block, derived from the increase in
    /// total address count.
    pub new: NewAddrCountVecs,
    /// Addresses that have received more than one output over their lifetime.
    pub reused: ReusedAddrVecs<M>,
    /// Addresses from which more than one output has been spent over their
    /// lifetime.
    pub respent: ReusedAddrVecs<M>,
    /// Addresses whose public key or spending script has appeared on-chain.
    /// P2PK and P2TR are exposed when funded; hashed script types become
    /// exposed when spent; P2A is excluded.
    pub exposed: ExposedAddrVecs<M>,
    /// Change in funded address count over the named trailing window, with the
    /// percentage change measured against the window's starting count.
    pub delta: DeltaVecs,
    pub avg_amount: AvgAmountVecs<M>,
    #[traversable(wrap = "indexes", rename = "funded")]
    /// Identity index for the persisted funded-address state table.
    pub funded_index: LazyVec<FundedAddrIndex, FundedAddrIndex, FundedAddrIndex, FundedAddrData>,
    #[traversable(wrap = "indexes", rename = "empty")]
    /// Identity index for the persisted empty-address state table.
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

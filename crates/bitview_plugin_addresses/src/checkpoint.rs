use bitview_cohort::AddressType;
use bitview_distribution::replay::validate_outputs;
use brk_error::Result;
use brk_types::{Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Stamp};

use crate::{
    Vecs,
    addr::AddressVecs,
    balance::Balances,
    state::{AddrStates, MinimalRealizedState, RealizedOps},
};

impl Vecs {
    /// Every height-indexed vec the block loop writes.
    pub(crate) fn loop_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        loop_vecs(&mut self.all, &mut self.types, &mut self.balances)
    }

    /// Every vec the block loop writes and checkpoints.
    pub(crate) fn stateful_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let Self {
            all,
            types,
            balances,
            state,
            ..
        } = self;
        let mut vecs = loop_vecs(all, types, balances);
        vecs.extend(state.par_iter_mut().collect::<Vec<_>>());
        vecs
    }

    /// Restart the metrics from the first block.
    pub(crate) fn reset_metrics(&mut self) -> Result<()> {
        self.loop_vecs_mut()
            .into_iter()
            .try_for_each(|v| v.any_truncate_if_needed_at(0))?;
        Ok(())
    }

    /// `None` after a version change, else the height every height-indexed
    /// output reaches; address state resumes by stamp instead.
    pub(crate) fn validate_state(&mut self, version: Version) -> Result<Option<usize>> {
        let caps_changed = self.caps.validate(version)?;
        let state_changed = self
            .state
            .par_iter_mut()
            .map(|v| v.any_validate_computed_version_or_reset(version))
            .try_reduce(|| false, |a, b| Ok(a || b))?;
        let outputs = validate_outputs(self.loop_vecs_mut().into_par_iter(), version)?;
        Ok(outputs.filter(|_| !caps_changed && !state_changed))
    }

    pub(crate) fn rollback_state(&mut self, start: usize) -> Result<usize> {
        let stamp = Stamp::from(Height::from(start));
        let mut stamps = self.state.rollback_before(stamp)?;
        stamps.push(self.caps.rollback_before(stamp)?);
        let recovered = usize::from(Height::from(stamps[0]).incremented());
        Ok(
            if recovered <= start && stamps.iter().all(|s| *s == stamps[0]) {
                recovered
            } else {
                0
            },
        )
    }

    pub(crate) fn restore_caps(&self, addrs: &mut AddrStates) -> Option<()> {
        let caps = self.caps.load()?;
        for (state, cap) in addrs
            .amount_range
            .iter_mut()
            .map(|s| &mut s.inner.realized)
            .zip(caps)
        {
            *state = MinimalRealizedState::from_cap(cap);
        }
        Some(())
    }

    pub(crate) fn save_caps(
        &mut self,
        addrs: &AddrStates,
        stamp: Stamp,
        with_changes: bool,
    ) -> Result<()> {
        self.caps.save(
            addrs
                .amount_range
                .iter()
                .map(|s| s.inner.realized.cap_raw()),
            stamp,
            with_changes,
        )?;
        Ok(())
    }
}

fn loop_vecs<'a>(
    all: &'a mut AddressVecs,
    types: &'a mut AddressType<AddressVecs>,
    balances: &'a mut Balances,
) -> Vec<&'a mut dyn AnyStoredVec> {
    let mut vecs: Vec<&mut dyn AnyStoredVec> = Vec::with_capacity(512);
    vecs.extend(all.loop_vecs_mut());
    for member in types.iter_mut() {
        vecs.extend(member.loop_vecs_mut());
    }
    vecs.extend(balances.vecs_mut());
    vecs
}

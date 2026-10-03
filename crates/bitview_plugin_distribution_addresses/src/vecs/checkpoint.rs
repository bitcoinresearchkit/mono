use bitview_plugin_distribution_common::replay::validate_outputs;
use brk_error::Result;
use brk_types::{Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Stamp};

use crate::{
    Vecs,
    state::{AddrStates, MinimalRealizedState, RealizedOps},
};

impl Vecs {
    /// Every vec the block loop writes and checkpoints.
    pub(crate) fn par_iter_stateful_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.addr_state
            .par_iter_mut()
            .chain(self.addrs.par_iter_stateful_height_mut())
            .chain(self.balances.par_iter_vecs_mut())
    }

    /// `None` after a version change, else the height every height-indexed
    /// output reaches; address state resumes by stamp instead.
    pub(crate) fn validate_state(&mut self, version: Version) -> Result<Option<usize>> {
        let caps_changed = self.caps.validate(version)?;
        let state_changed = self
            .addr_state
            .par_iter_mut()
            .map(|v| v.any_validate_computed_version_or_reset(version))
            .try_reduce(|| false, |a, b| Ok(a || b))?;
        let outputs = validate_outputs(
            self.addrs
                .par_iter_stateful_height_mut()
                .chain(self.balances.par_iter_vecs_mut()),
            version,
        )?;
        Ok(outputs.filter(|_| !caps_changed && !state_changed))
    }

    pub(crate) fn rollback_state(&mut self, start: usize) -> Result<usize> {
        let stamp = Stamp::from(Height::from(start));
        let mut stamps = self.addr_state.rollback_before(stamp)?;
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

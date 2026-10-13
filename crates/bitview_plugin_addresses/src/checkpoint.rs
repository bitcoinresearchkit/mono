use bitview_cohort::AddressType;
use bitview_distribution::replay::validate_outputs;
use bitview_plugin_indexer::Indexer;
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
        let stable = self.state.stable();
        let mut written = stable.write();
        let state_changed = self
            .state
            .par_iter_mut()
            .map(|v| v.any_validate_computed_version_or_reset(version))
            .try_reduce(|| false, |a, b| Ok(a || b))?;
        if state_changed {
            *written = None;
        }
        drop(written);
        let outputs = validate_outputs(self.loop_vecs_mut().into_par_iter(), version)?;
        Ok(outputs.filter(|_| !caps_changed && !state_changed))
    }

    /// Drops the address state an update that failed before publishing left staged.
    pub(crate) fn discard_unpublished_state(&mut self) {
        if self.unpublished_state.take().is_some() {
            self.state.reset_unsaved();
        }
    }

    /// Rolls the address state (rewritten in place) back below `start`, to the newest saved
    /// change under it, lowering the publication there first, and writes it: readers, who take its
    /// lock (`written`) before they pin, read it there at once. Returns where it lands: `start`
    /// when nothing lies past it, 0 when the saved changes do not reach below it or the vectors land
    /// apart (left unwritten for the caller to reset).
    pub fn roll_back_to(
        &mut self,
        start: usize,
        indexer: &Indexer,
        written: &mut Option<Height>,
    ) -> Result<usize> {
        self.discard_unpublished_state();
        let current = usize::from(self.state.max_stamped_len()).max(self.caps.end());
        if start == 0 || start >= current {
            return Ok(start);
        }
        let stamp = Stamp::from(Height::from(start));
        if let Some(landing) = self.state.rollback_landing(stamp)?
            && usize::from(landing) < start
        {
            indexer.lower_publication_to(landing);
        }
        // The restored values sit in the write buffers until written.
        *written = None;
        let mut stamps = self.state.rollback_before(stamp)?;
        stamps.push(self.caps.rollback_before(stamp)?);
        let landing = stamps[0];
        let recovered = usize::from(Height::from(landing).incremented());
        if recovered > start || stamps.iter().any(|s| *s != landing) {
            return Ok(0);
        }
        self.write_state(landing, false, written)?;
        Ok(recovered)
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

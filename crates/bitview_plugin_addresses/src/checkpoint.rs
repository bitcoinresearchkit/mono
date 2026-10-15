use bitview_distribution::replay::{LiveState, Replay, validate_outputs};
use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Stamp};

use crate::{
    Vecs,
    state::{AddrStates, MinimalRealizedState, RealizedOps},
};

impl Replay for Vecs {
    type State = AddrStates;

    fn live(&mut self) -> &mut Option<LiveState<AddrStates>> {
        &mut self.live
    }

    /// Address state resumes by stamp; the other outputs by length.
    fn validate(&mut self, version: Version) -> Result<Option<usize>> {
        let caps_changed = self.caps.validate(version)?;
        let stable = self.state.stable();
        let mut stable = stable.write();
        let state_changed = self
            .state
            .par_iter_mut()
            .map(|v| v.any_validate_computed_version_or_reset(version))
            .try_reduce(|| false, |a, b| Ok(a || b))?;
        if state_changed {
            stable.written = None;
            stable.changed = None;
        }
        drop(stable);
        let outputs = validate_outputs(self.loop_vecs_mut(), version)?;
        Ok(outputs.filter(|_| !caps_changed && !state_changed))
    }

    fn checkpoint_len(&self) -> usize {
        usize::from(self.state.min_stamped_len()).min(self.caps.end())
    }

    /// Also truncates the outputs where the state lands: the block loop appends from there.
    fn roll_back(&mut self, start: usize, indexer: &Indexer) -> Result<usize> {
        let landing = self.roll_back_to(start, indexer)?;
        if landing > 0 {
            indexer.rewrite_from(Height::from(landing), || self.truncate_outputs(landing))?;
        }
        Ok(landing)
    }

    fn restore(&self, addrs: &mut AddrStates, height: Height) -> Option<()> {
        addrs.restore(&self.balances, &self.types, height)?;
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

    /// Restarts the address state, the caps and the metrics from the first block.
    fn reset(&mut self) -> Result<()> {
        self.state.reset()?;
        self.caps.reset()?;
        self.truncate_outputs(0)
    }
}

impl Vecs {
    fn truncate_outputs(&mut self, len: usize) -> Result<()> {
        self.loop_vecs_mut()
            .try_for_each(|v| v.any_truncate_if_needed_at(len))?;
        Ok(())
    }

    /// Every height-indexed vec the block loop writes: all the members' and bands' stored vecs.
    pub(crate) fn loop_vecs_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        let Self {
            all,
            types,
            balances,
            ..
        } = self;
        rayon::iter::once(&mut **all)
            .chain(types.par_iter_mut())
            .flat_map_iter(|member| member.iter_any_stored_mut())
            .chain(
                balances
                    .bands
                    .par_iter_mut()
                    .flat_map_iter(|band| band.iter_any_stored_mut()),
            )
            .chain(rayon::iter::once(
                &mut balances.capital as &mut dyn AnyStoredVec,
            ))
    }

    /// Drops the address state an update that failed before publishing left staged.
    pub(crate) fn discard_unpublished_state(&mut self) {
        if self.unpublished_state.take().is_some() {
            self.state.reset_unsaved();
        }
    }

    /// Rolls the address state (rewritten in place) and the caps back below `start`, to the
    /// newest saved change under it, and writes the state. The publication is lowered to the
    /// landing first; the state's lock (`stable`, which readers take before they pin) is held
    /// only for the rollback and the write, never while earlier pins drain: meanwhile readers
    /// find the state at another length than their pin and answer at once that it is updating.
    /// Returns where it lands: `start` when nothing lies past it, or 0 when the saved changes do
    /// not reach below it (a reorg deeper than the protected tail): nothing is rolled back, and
    /// the caller's reset rebuilds the plugin.
    pub fn roll_back_to(&mut self, start: usize, indexer: &Indexer) -> Result<usize> {
        self.discard_unpublished_state();
        let current = usize::from(self.state.max_stamped_len()).max(self.caps.end());
        if start == 0 || start >= current {
            return Ok(start);
        }
        let stamp = Stamp::from(Height::from(start));
        let landing = match (
            self.state.rollback_landing(stamp)?,
            self.caps.rollback_landing(stamp)?,
        ) {
            (Some(state), Some(caps)) if usize::from(state) == caps && caps <= start => state,
            _ => return Ok(0),
        };
        indexer.rewrite_from(landing, || {
            let stable = self.state.stable();
            let mut stable = stable.write();
            // The restored values sit in the write buffers until written.
            stable.written = None;
            stable.changed = None;
            let landing_stamp = Stamp::from(landing.decremented().unwrap_or_default());
            let mut stamps = self.state.rollback_before(stamp)?;
            stamps.push(self.caps.rollback_before(stamp)?);
            debug_assert!(
                stamps.iter().all(|s| *s == landing_stamp),
                "rollback lands where its preflight did"
            );
            self.write_state(landing_stamp, false, &mut stable)
        })?;
        Ok(usize::from(landing))
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

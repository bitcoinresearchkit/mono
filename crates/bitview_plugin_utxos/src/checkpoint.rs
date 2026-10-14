use bitview_distribution::replay::{LiveState, Replay, validate_outputs};
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_types::{Height, Version};
use rayon::prelude::*;
use vecdb::Stamp;

use crate::{
    Vecs,
    state::{MinimalRealizedState, RealizedOps, UTXOStates},
};

impl Replay for Vecs {
    type State = UTXOStates;

    fn live(&mut self) -> &mut Option<LiveState<UTXOStates>> {
        &mut self.live
    }

    fn validate(&mut self, version: Version) -> Result<Option<usize>> {
        let caps_changed = self.caps.validate(version)?;
        let outputs = validate_outputs(self.cohorts.par_iter_vecs_mut(), version)?;
        Ok(outputs.filter(|_| !caps_changed))
    }

    fn checkpoint_len(&self) -> usize {
        self.caps.end()
    }

    /// Rolls the caps back to the newest saved change at or below `start`. When the saved changes
    /// do not reach it (a deep reorg, or a catch-up's unsaved chunk) nothing is rolled back and
    /// the publication is lowered to genesis, so readers are off every row the reset drops. Also
    /// truncates the outputs where the caps land: the block loop appends from there.
    fn roll_back(&mut self, start: usize, indexer: &Indexer) -> Result<usize> {
        let landing = if start >= self.caps.end() {
            start
        } else {
            let stamp = Stamp::from(Height::from(start));
            match self.caps.rollback_landing(stamp)? {
                Some(landing) if landing <= start => {
                    let landed = self.caps.rollback_before(stamp)?;
                    debug_assert_eq!(usize::from(Height::from(landed).incremented()), landing);
                    landing
                }
                _ => {
                    indexer.lower_publication_to(Height::ZERO);
                    0
                }
            }
        };
        self.truncate_outputs(landing)?;
        Ok(landing)
    }

    fn restore(&self, states: &mut UTXOStates, height: Height) -> Option<()> {
        states.restore(&self.cohorts, height)?;
        for (state, cap) in states
            .amount_range
            .iter_mut()
            .chain(states.type_.iter_mut())
            .zip(self.caps.load()?)
        {
            state.realized = MinimalRealizedState::from_cap(cap);
        }
        Some(())
    }

    fn reset(&mut self) -> Result<()> {
        self.caps.reset()?;
        self.truncate_outputs(0)
    }
}

impl Vecs {
    fn truncate_outputs(&mut self, len: usize) -> Result<()> {
        self.cohorts
            .par_iter_vecs_mut()
            .try_for_each(|v| v.any_truncate_if_needed_at(len))?;
        Ok(())
    }

    pub(crate) fn save_caps(
        &mut self,
        states: &UTXOStates,
        stamp: Stamp,
        with_changes: bool,
    ) -> Result<()> {
        self.caps.save(
            states
                .amount_range
                .iter()
                .chain(states.type_.iter())
                .map(|s| s.realized.cap_raw()),
            stamp,
            with_changes,
        )
    }
}

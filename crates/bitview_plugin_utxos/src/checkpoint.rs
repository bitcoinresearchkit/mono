use bitview_distribution::replay::{LiveState, Replay, validate_outputs};
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_types::{Height, Version};
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

    fn roll_back(&mut self, start: usize, _indexer: &Indexer) -> Result<usize> {
        if start >= self.caps.end() {
            return Ok(start);
        }
        let stamp = self
            .caps
            .rollback_before(Stamp::from(Height::from(start)))?;
        let end = usize::from(Height::from(stamp).incremented());
        Ok(if end <= start { end } else { 0 })
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
        self.caps.reset()
    }
}

impl Vecs {
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

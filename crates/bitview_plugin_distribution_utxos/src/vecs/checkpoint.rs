use crate::{
    Vecs,
    state::{MinimalRealizedState, RealizedOps, UTXOStates},
};
use brk_error::Result;
use brk_types::{Height, Version};
use rayon::prelude::*;
use vecdb::Stamp;
impl Vecs {
    pub(crate) fn validate_state(&mut self, version: Version) -> Result<bool> {
        let caps_changed = self.caps.validate(version)?;
        let vecs_changed = self
            .cohorts
            .par_iter_vecs_mut()
            .map(|v| v.any_validate_computed_version_or_reset(version))
            .try_reduce(|| false, |a, b| Ok(a || b))?;
        Ok(caps_changed || vecs_changed)
    }
    pub(crate) fn rollback_state(&mut self, start: usize) -> Result<usize> {
        let stamp = self
            .caps
            .rollback_before(Stamp::from(Height::from(start)))?;
        let end = usize::from(Height::from(stamp).incremented());
        Ok(if end <= start { end } else { 0 })
    }
    pub(crate) fn restore_caps(&self, states: &mut UTXOStates) -> Option<()> {
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

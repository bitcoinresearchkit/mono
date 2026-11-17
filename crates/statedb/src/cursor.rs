use crate::{BlockDiff, Creations, Diffs, Spends, State};
use std::io::Result;

/// Sequential replay over one validated prefix, with bounded read windows.
/// The caller owns the state; analytics borrow it instead of copying its amounts.
pub struct Cursor<'a> {
    state: &'a mut State,
    diffs: Diffs<'a>,
}

impl<'a> Cursor<'a> {
    /// `state` must continue the producers' chain: its blocks are not read back.
    pub(crate) fn new(
        state: &'a mut State,
        end: usize,
        spends: &'a Spends,
        creations: &'a Creations,
    ) -> Result<Self> {
        let hash = (!state.is_empty()).then(|| state.hash());
        let diffs = Diffs::new(state.len(), hash, end, spends, creations)?;
        Ok(Self { state, diffs })
    }

    pub fn state(&self) -> &State {
        self.state
    }

    /// Applies one whole block and returns its changes with the state after it; an error leaves
    /// the state unchanged. None marks the published end.
    pub fn advance(&mut self) -> Result<Option<(BlockDiff<'_>, &State)>> {
        let Some(diff) = self.diffs.advance()? else {
            return Ok(None);
        };
        self.state.apply(&diff);
        Ok(Some((diff, self.state)))
    }
}

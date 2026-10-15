use crate::{BlockDiff, Creations, Diffs, Spends, State, util::invalid};
use std::io::Result;

/// Sequential replay over one validated prefix, with bounded read windows.
/// The caller owns the state; analytics borrow it instead of copying its amounts.
pub struct Cursor<'a> {
    state: &'a mut State,
    diffs: Diffs<'a>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(
        state: &'a mut State,
        end: usize,
        spends: &'a Spends,
        creations: &'a Creations,
    ) -> Result<Self> {
        let hash = if state.is_empty() {
            None
        } else {
            let last = state.len() - 1;
            if state.hash() != spends.hash(last)? || state.hash() != creations.hash(last)? {
                return Err(invalid("replay state does not match producer chain"));
            }
            Some(state.hash())
        };
        let diffs = Diffs::new(state.len(), hash, end, spends, creations)?;
        Ok(Self { state, diffs })
    }

    pub fn state(&self) -> &State {
        self.state
    }

    /// Applies one whole block and returns its changes; an error leaves the state unchanged.
    /// None marks the published end.
    pub fn advance(&mut self) -> Result<Option<BlockDiff<'_>>> {
        let diff = self.diffs.advance()?;
        if let Some(diff) = &diff {
            self.state.apply(diff);
        }
        Ok(diff)
    }
}

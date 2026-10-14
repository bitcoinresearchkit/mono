use crate::{BlockDiff, Creations, Spends, State, journal_reader::JournalReader, util::invalid};
use std::io::Result;

/// Sequential replay over one validated prefix, with bounded read windows.
/// The caller owns the state; analytics borrow it instead of copying its amounts.
pub struct Cursor<'a> {
    state: &'a mut State,
    spends: JournalReader<'a>,
    creations: JournalReader<'a>,
    end: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(
        state: &'a mut State,
        end: usize,
        spends: &'a Spends,
        creations: &'a Creations,
    ) -> Result<Self> {
        if state.len() > end || end > spends.end().min(creations.end()) {
            return Err(invalid("invalid replay range"));
        }
        if !state.is_empty()
            && (state.hash() != spends.hash(state.len() - 1)?
                || state.hash() != creations.hash(state.len() - 1)?)
        {
            return Err(invalid("replay state does not match producer chain"));
        }
        Ok(Self {
            state,
            spends: spends.cursor(end),
            creations: creations.cursor(end),
            end,
        })
    }

    pub fn state(&self) -> &State {
        self.state
    }

    /// Applies one whole block and returns its changes; an error leaves the state unchanged.
    /// None marks the published end.
    pub fn advance(&mut self) -> Result<Option<BlockDiff<'_>>> {
        if self.state.len() == self.end {
            return Ok(None);
        }
        let height = self.state.len();
        let (parent, spent) = self.spends.read(height)?;
        // A rewind racing this replay (readers hold their own file handles) can hand it another
        // chain's block: the read must continue the state's chain before any row applies.
        if height > 0 && parent.get(..32) != Some(self.state.hash().as_slice()) {
            return Err(invalid("replay state does not match producer chain"));
        }
        let diff = BlockDiff::decode(height, spent, self.creations.read(height)?.1)?;
        self.state.apply(&diff);
        Ok(Some(diff))
    }
}

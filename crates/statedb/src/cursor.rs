use crate::{Amount, BlockDiff, Creations, Spends, State, util::invalid};
use std::io::Result;

/// Sequential replay over one validated prefix, with reusable decode and rollback buffers.
/// The caller owns the state; analytics borrow it instead of copying its amounts.
pub struct Cursor<'a> {
    state: &'a mut State,
    spends: &'a Spends,
    creations: &'a Creations,
    end: usize,
    diff: BlockDiff,
    scratch: Vec<(usize, Amount)>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(
        state: &'a mut State,
        end: usize,
        spends: &'a Spends,
        creations: &'a Creations,
    ) -> Result<Self> {
        let start = spends.start().max(creations.start());
        if state.len() < start || state.len() > end || end > spends.len().min(creations.len()) {
            return Err(invalid("invalid replay range"));
        }
        if state.len() > start
            && (state.hash() != spends.hash(state.len() - 1)?
                || state.hash() != creations.read(state.len() - 1)?.0)
        {
            return Err(invalid("replay state does not match producer chain"));
        }
        Ok(Self {
            state,
            spends,
            creations,
            end,
            diff: BlockDiff::default(),
            scratch: Vec::new(),
        })
    }

    pub fn state(&self) -> &State {
        self.state
    }

    /// Applies one whole block atomically and returns its changes. None marks the published end.
    pub fn advance(&mut self) -> Result<Option<&BlockDiff>> {
        if self.state.len() == self.end {
            return Ok(None);
        }
        self.diff
            .read(self.state.len(), self.spends, self.creations)?;
        self.state.apply(
            self.diff.hash,
            self.diff.created,
            self.diff.removed(),
            &mut self.scratch,
        )?;
        Ok(Some(&self.diff))
    }
}

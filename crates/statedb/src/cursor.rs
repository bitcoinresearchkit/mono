use crate::{
    Amount, BlockDiff, Creations, Spends, State, journal_reader::JournalReader, util::invalid,
};
use std::io::Result;

/// Sequential replay over one validated prefix, with bounded read windows and reusable rollback scratch.
/// The caller owns the state; analytics borrow it instead of copying its amounts.
pub struct Cursor<'a> {
    state: &'a mut State,
    spends: JournalReader<'a>,
    creations: JournalReader<'a>,
    end: usize,
    scratch: Vec<(usize, Amount)>,
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
            scratch: Vec::new(),
        })
    }

    pub fn state(&self) -> &State {
        self.state
    }

    /// Applies one whole block atomically and returns its changes. None marks the published end.
    pub fn advance(&mut self) -> Result<Option<BlockDiff<'_>>> {
        if self.state.len() == self.end {
            return Ok(None);
        }
        let height = self.state.len();
        let diff = BlockDiff::decode(
            height,
            self.spends.read(height)?,
            self.creations.read(height)?,
        )?;
        self.state.apply(
            diff.hash,
            diff.created,
            diff.removed(),
            diff.removed_total,
            &mut self.scratch,
        )?;
        Ok(Some(diff))
    }
}

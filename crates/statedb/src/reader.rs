use crate::{
    Amount, BlockDiff, Creations, Cursor, History, Spends, State, snapshot::Snapshot, util::invalid,
};
use std::{io::Result, ops::Range};

/// An immutable view of one published prefix. Its borrows prevent producer mutation.
pub struct Reader<'a> {
    pub(crate) history: &'a History,
    pub(crate) spends: &'a Spends,
    pub(crate) creations: &'a Creations,
    pub(crate) end: usize,
    pub(crate) latest: Option<Snapshot>,
}

impl Reader<'_> {
    /// Earliest reconstructible state length; zero means genesis is available.
    pub fn start(&self) -> usize {
        self.spends.start().max(self.creations.start())
    }

    pub fn versions(&self) -> (u64, u64) {
        (self.spends.version(), self.creations.version())
    }

    /// Checks the prefix identity before reusing resident analytical state.
    pub fn matches(&self, state: &State) -> Result<bool> {
        let len = state.len();
        if len < self.start() || len > self.end {
            return Ok(false);
        }
        Ok(len == self.start()
            || (state.hash() == self.spends.hash(len - 1)?
                && state.hash() == self.creations.read(len - 1)?.0))
    }

    pub fn len(&self) -> usize {
        self.end
    }

    pub fn is_empty(&self) -> bool {
        self.end == 0
    }

    pub fn state_at(&self, end: usize) -> Result<State> {
        if end > self.end {
            return Err(invalid("height exceeds published origin history"));
        }
        self.history
            .restore(end, self.spends, self.creations, self.latest.as_ref())
    }

    /// Decodes one published block into reusable scratch. Use it only after a successful read.
    pub fn read_block(&self, height: usize, diff: &mut BlockDiff) -> Result<()> {
        if height >= self.end {
            return Err(invalid("height exceeds published origin history"));
        }
        diff.read(height, self.spends, self.creations)
    }

    pub fn cursor<'a>(&'a self, state: &'a mut State) -> Result<Cursor<'a>> {
        Cursor::new(state, self.end, self.spends, self.creations)
    }

    pub fn replay(
        &self,
        state: &mut State,
        end: usize,
        visit: impl FnMut(usize, Amount) -> Result<()>,
    ) -> Result<()> {
        self.validate_range(&(state.len()..end))?;
        History::replay(state, end, self.spends, self.creations, visit)
    }

    fn validate_range(&self, range: &Range<usize>) -> Result<()> {
        if range.start > range.end
            || range.start < self.spends.start().max(self.creations.start())
            || range.end > self.end
        {
            return Err(invalid("range outside published origin history"));
        }
        Ok(())
    }
}

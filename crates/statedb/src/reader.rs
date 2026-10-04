use crate::{Creations, Cursor, History, Spends, State, snapshot::Snapshot, util::invalid};
use std::io::Result;

/// An immutable view of one published prefix. Its borrows prevent producer mutation.
pub struct Reader<'a> {
    pub(crate) history: &'a History,
    pub(crate) spends: &'a Spends,
    pub(crate) creations: &'a Creations,
    pub(crate) end: usize,
    pub(crate) latest: Option<Snapshot>,
}

impl Reader<'_> {
    pub fn versions(&self) -> (u64, u64) {
        (self.spends.version(), self.creations.version())
    }

    /// Checks the prefix identity before reusing resident analytical state.
    pub fn matches(&self, state: &State) -> Result<bool> {
        let len = state.len();
        if len > self.end {
            return Ok(false);
        }
        Ok(len == 0
            || (state.hash() == self.spends.hash(len - 1)?
                && state.hash() == self.creations.hash(len - 1)?))
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

    pub fn cursor<'a>(&'a self, state: &'a mut State) -> Result<Cursor<'a>> {
        Cursor::new(state, self.end, self.spends, self.creations)
    }
}

use crate::{
    Creations, Cursor, Diffs, History, Spends, State, history::continues, snapshot::Snapshot,
    util::invalid,
};
use std::{io::Result, sync::Mutex};

/// An immutable view of one published prefix. Its borrows prevent producer mutation.
pub struct Reader<'a> {
    pub(crate) history: &'a History,
    pub(crate) spends: &'a Spends,
    pub(crate) creations: &'a Creations,
    pub(crate) end: usize,
    /// Tip snapshots the producers carry, newest first.
    pub(crate) tips: Vec<Snapshot>,
    /// The last prefix found to match (length, hash): consumers resuming from one height share it.
    pub(crate) checked: Mutex<Option<(usize, [u8; 32])>>,
}

impl Reader<'_> {
    pub fn versions(&self) -> (u64, u64) {
        (self.spends.version(), self.creations.version())
    }

    /// Whether `state`, retained from an earlier update, ends at `start` on this history's chain:
    /// a consumer then resumes from it instead of restoring the state at `start`.
    pub fn resumes(&self, state: &State, start: usize) -> Result<bool> {
        Ok(state.len() == start && self.matches(state)?)
    }

    /// Checks the prefix identity before reusing resident analytical state.
    pub(crate) fn matches(&self, state: &State) -> Result<bool> {
        let prefix = (state.len(), state.hash());
        if prefix.0 > self.end {
            return Ok(false);
        }
        let mut checked = self.checked.lock().unwrap();
        if *checked == Some(prefix) {
            return Ok(true);
        }
        let matches = continues(prefix.0, prefix.1, self.spends, self.creations)?;
        if matches {
            *checked = Some(prefix);
        }
        Ok(matches)
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
            .restore(end, self.spends, self.creations, &self.tips)
    }

    pub fn cursor<'a>(&'a self, state: &'a mut State) -> Result<Cursor<'a>> {
        if !self.matches(state)? {
            return Err(invalid("replay state does not match producer chain"));
        }
        Cursor::new(state, self.end, self.spends, self.creations)
    }

    /// The changes of blocks `first..`, read without restoring the state before them.
    pub fn diffs(&self, first: usize) -> Result<Diffs<'_>> {
        Diffs::new(first, None, self.end, self.spends, self.creations)
    }
}

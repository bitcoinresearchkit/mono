use std::{io::Result, path::Path};

use crate::{Creations, History, Reader, Spends};

/// Read-only files of the published history. Snapshot writes never change what an open view reads;
/// the caller keeps producers from rewinding below the ceiling it reads at. No writer locks,
/// truncation, directory creation, or full journal-index copies occur here.
pub struct View {
    history: History,
    spends: Spends,
    creations: Creations,
}

impl View {
    pub fn open(history_path: &Path, spends_path: &Path, creations_path: &Path) -> Result<Self> {
        Ok(Self {
            history: History::open_reader(history_path)?,
            spends: Spends::open_reader(spends_path)?,
            creations: Creations::open_reader(creations_path)?,
        })
    }

    /// A reader of at most `ceiling` blocks (the caller's published height) that reads nothing
    /// past them.
    pub fn reader(&self, ceiling: usize) -> Result<Reader<'_>> {
        self.history
            .reader_within(&self.spends, &self.creations, ceiling)
    }
}

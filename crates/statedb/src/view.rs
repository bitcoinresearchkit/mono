use std::{io::Result, path::Path};

use crate::{Creations, History, Reader, Spends};

/// Read-only files for an externally pinned publication. The caller must prevent
/// producer rewinds until its reader/state capture completes. No writer locks,
/// truncation, directory creation, or full journal-index copies occur here.
pub struct View {
    history: History,
    spends: Spends,
    creations: Creations,
}

impl View {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            history: History::open_reader(path),
            spends: Spends::open_reader(path)?,
            creations: Creations::open_reader(path)?,
        })
    }

    pub fn reader(&self) -> Result<Reader<'_>> {
        self.history.reader(&self.spends, &self.creations)
    }
}

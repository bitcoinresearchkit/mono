// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    fs::{File, TryLockError},
    path::Path,
    sync::Arc,
};

use log::{debug, error};

use crate::{Error, Result};

/// The database lock, held while any clone lives: closing the file releases it.
pub type LockedFileGuard = Arc<File>;

/// Opens (creating it if needed) and locks the database lock file.
pub fn lock(path: &Path) -> Result<LockedFileGuard> {
    debug!("Acquiring database lock at {}", path.display());

    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;

    file.try_lock().map_err(|e| match e {
        TryLockError::Error(e) => {
            error!("Failed to acquire database lock - if this is expected, you can try opening again (maybe wait a little)");
            Error::Io(e)
        }
        TryLockError::WouldBlock => Error::Locked,
    })?;

    Ok(Arc::new(file))
}

/// An iterator holding the database lock until the tables it reads are released.
pub struct LockedIter<I> {
    // Declared first so it drops first: its tables release before the lock.
    iter: I,
    _lock: LockedFileGuard,
}

impl<I> LockedIter<I> {
    /// Ties `iter` to the database lock.
    pub fn new(iter: I, lock: &LockedFileGuard) -> Self {
        Self {
            iter,
            _lock: lock.clone(),
        }
    }
}

impl<I: Iterator> Iterator for LockedIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for LockedIter<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

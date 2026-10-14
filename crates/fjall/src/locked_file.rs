// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    fs::{File, OpenOptions, TryLockError},
    io::ErrorKind,
    path::Path,
    sync::Arc,
};

use log::{debug, error, warn};

use crate::{Error, Result};

struct LockedFileGuardInner(File);

impl Drop for LockedFileGuardInner {
    fn drop(&mut self) {
        debug!("Unlocking database lock");

        self.0
            .unlock()
            .inspect_err(|e| {
                warn!("Failed to unlock database lock: {e:?}");
            })
            .ok();
    }
}

#[derive(Clone)]
#[expect(unused)]
pub struct LockedFileGuard(Arc<LockedFileGuardInner>);

impl LockedFileGuard {
    pub fn create_new(path: &Path) -> Result<Self> {
        debug!("Acquiring database lock at {}", path.display());

        let file = match File::create_new(path) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                OpenOptions::new().read(true).write(true).open(path)?
            }
            e => e?,
        };

        file.try_lock().map_err(|e| match e {
            TryLockError::Error(e) => {
                error!("Failed to acquire database lock - if this is expected, you can try opening again (maybe wait a little)");
                Error::Io(e)
            }
            TryLockError::WouldBlock => Error::Locked,
        })?;

        Ok(Self(Arc::new(LockedFileGuardInner(file))))
    }
}

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use inner::HeaderInner;
use parking_lot::RwLock;
use rawdb::Region;

use super::Format;
use crate::{Error, Result, Stamp, Version};
use write_guard::WriteGuard;

pub mod inner;
mod write_guard;

const HEADER_VERSION: Version = Version::TWO;
pub const HEADER_OFFSET: usize = size_of::<HeaderInner>();

#[derive(Debug, Clone)]
pub struct Header {
    inner: Arc<RwLock<HeaderInner>>,
    modified: bool,
    write_failed: Arc<AtomicBool>,
}

impl Header {
    pub(crate) fn create_and_write(
        region: &Region,
        vec_version: Version,
        format: Format,
    ) -> Result<Self> {
        let inner = HeaderInner::create_and_write(region, vec_version, format)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
            modified: false,
            write_failed: Arc::new(AtomicBool::new(false)),
        })
    }

    pub(crate) fn import_and_verify(
        region: &Region,
        vec_version: Version,
        format: Format,
    ) -> Result<Self> {
        let inner = HeaderInner::import_and_verify(region, vec_version, format)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
            modified: false,
            write_failed: Arc::new(AtomicBool::new(false)),
        })
    }

    pub(crate) fn update_stamp(&mut self, stamp: Stamp) {
        self.assert_writable();
        let mut inner = self.inner.write();
        if inner.stamp != stamp {
            self.modified = true;
            inner.stamp = stamp;
        }
    }

    pub(crate) fn update_computed_version(&mut self, computed_version: Version) {
        self.assert_writable();
        let mut inner = self.inner.write();
        if inner.computed_version != computed_version {
            self.modified = true;
            inner.computed_version = computed_version;
        }
    }

    #[inline(always)]
    pub(crate) fn modified(&self) -> bool {
        self.modified
    }

    #[inline(always)]
    pub fn vec_version(&self) -> Version {
        self.inner.read().vec_version
    }

    #[inline(always)]
    pub fn computed_version(&self) -> Version {
        self.inner.read().computed_version
    }

    /// Computed sources carry their provenance; raw sources carry their schema.
    #[inline(always)]
    pub(crate) fn source_version(&self) -> Version {
        let inner = self.inner.read();
        if inner.computed_version == Version::ZERO {
            inner.vec_version
        } else {
            inner.computed_version
        }
    }

    #[inline(always)]
    pub(crate) fn stamp(&self) -> Stamp {
        self.inner.read().stamp
    }

    pub(crate) fn write(&mut self, region: &Region) -> Result<()> {
        let guard = self.begin_write()?;
        self.inner.read().write(region)?;
        self.modified = false;
        guard.finish(Ok(()))
    }

    pub(crate) fn check_writable(&self) -> Result<()> {
        if self.write_failed.load(Ordering::Relaxed) {
            Err(Error::WriteFailed)
        } else {
            Ok(())
        }
    }

    pub(crate) fn assert_writable(&self) {
        self.check_writable()
            .expect("vector cannot continue after a failed write");
    }

    pub(crate) fn begin_write(&self) -> Result<WriteGuard> {
        WriteGuard::new(&self.write_failed)
    }
}

use std::sync::Arc;

use parking_lot::RwLock;
use rawdb::Region;

mod inner;

use inner::HeaderInner;

use crate::{Stamp, Version};

use super::Format;

const HEADER_VERSION: Version = Version::TWO;
pub const HEADER_OFFSET: usize = size_of::<HeaderInner>();

#[derive(Debug, Clone)]
pub struct Header {
    inner: Arc<RwLock<HeaderInner>>,
    modified: bool,
}

impl Header {
    pub fn create_and_write(
        region: &Region,
        vec_version: Version,
        format: Format,
    ) -> crate::Result<Self> {
        let inner = HeaderInner::create_and_write(region, vec_version, format)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
            modified: false,
        })
    }

    pub fn import_and_verify(
        region: &Region,
        vec_version: Version,
        format: Format,
    ) -> crate::Result<Self> {
        let inner = HeaderInner::import_and_verify(region, vec_version, format)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
            modified: false,
        })
    }

    pub fn update_stamp(&mut self, stamp: Stamp) {
        let mut inner = self.inner.write();
        if inner.stamp != stamp {
            self.modified = true;
            inner.stamp = stamp;
        }
    }

    pub fn update_computed_version(&mut self, computed_version: Version) {
        let mut inner = self.inner.write();
        if inner.computed_version != computed_version {
            self.modified = true;
            inner.computed_version = computed_version;
        }
    }

    #[inline(always)]
    pub fn modified(&self) -> bool {
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

    #[inline(always)]
    pub fn stamp(&self) -> Stamp {
        self.inner.read().stamp
    }

    pub fn write(&mut self, region: &Region) -> crate::Result<()> {
        self.inner.read().write(region)?;
        self.modified = false;
        Ok(())
    }
}

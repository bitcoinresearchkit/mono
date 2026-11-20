use std::sync::Arc;

use axum::body::Bytes;
use bitview_query::{Error, Result};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// One admitted response, from allocation until its body is sent.
pub struct RawBodyPermit {
    _permit: OwnedSemaphorePermit,
}

impl RawBodyPermit {
    // Eight concurrent responses per budget: raw blocks are at most 4 MB each (32 MB in all),
    // chain JSON can be larger.
    pub const CAPACITY: usize = 8;

    /// Bodies at least this large keep their permit until sent; smaller ones release it once
    /// built, so slow clients of small responses can't hold every permit. Build concurrency
    /// stays bounded either way.
    const RETAINED_FROM: usize = 256 * 1024;

    pub fn retains(len: usize) -> bool {
        len >= Self::RETAINED_FROM
    }

    pub fn try_acquire(budget: &Arc<Semaphore>) -> Option<Self> {
        let _permit = budget.clone().try_acquire_owned().ok()?;
        Some(Self { _permit })
    }

    pub async fn acquire(budget: &Arc<Semaphore>) -> Result<Self> {
        let _permit = budget
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| Error::Internal("response admission closed"))?;
        Ok(Self { _permit })
    }

    /// `bytes`, holding this permit until they are sent.
    pub fn bytes(self, bytes: Bytes) -> Bytes {
        Bytes::from_owner(Retained {
            bytes,
            _permit: self,
        })
    }
}

/// A body and the permit dropped with it.
struct Retained {
    bytes: Bytes,
    _permit: RawBodyPermit,
}

impl AsRef<[u8]> for Retained {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

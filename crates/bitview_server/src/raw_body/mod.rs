use std::sync::Arc;

use axum::body::Bytes;
use bitview_query::{Error, Result};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use bytes::RetainedBytes;

mod bytes;

/// One admitted response, from allocation until its body is sent.
#[derive(Clone)]
pub struct RawBodyPermit {
    _permit: Arc<OwnedSemaphorePermit>,
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
        budget.clone().try_acquire_owned().ok().map(|permit| Self {
            _permit: Arc::new(permit),
        })
    }

    pub async fn acquire(budget: &Arc<Semaphore>) -> Result<Self> {
        let permit = budget
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| Error::Internal("response admission closed"))?;
        Ok(Self {
            _permit: Arc::new(permit),
        })
    }

    /// `bytes`, holding this permit until they are sent.
    pub fn bytes(&self, bytes: Bytes) -> Bytes {
        Bytes::from_owner(RetainedBytes {
            bytes,
            _permit: self.clone(),
        })
    }
}

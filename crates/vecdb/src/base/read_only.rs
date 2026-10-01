use std::{marker::PhantomData, sync::Arc};

use rawdb::Region;

use crate::{VecIndex, VecValue, Version};

use super::{Header, SharedLen};

/// Read-only core of a stored vector — the minimal state needed for disk reads.
///
/// Contains region (I/O), shared length (bounds), name/header (metadata).
/// No pushed buffers, no rollback state.
#[derive(Debug, Clone)]
pub struct ReadOnlyBaseVec<I, T> {
    pub region: Region,
    pub stored_len: SharedLen,
    pub name: Arc<str>,
    pub header: Header,
    pub phantom: PhantomData<(I, T)>,
}

impl<I, T> ReadOnlyBaseVec<I, T>
where
    I: VecIndex,
    T: VecValue,
{
    #[inline(always)]
    pub fn region(&self) -> &Region {
        &self.region
    }

    #[inline(always)]
    pub fn header(&self) -> &Header {
        &self.header
    }

    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline(always)]
    pub fn stored_len(&self) -> usize {
        self.stored_len.get()
    }

    /// For read-only vecs, len == stored_len (no pushed buffer).
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.stored_len.get()
    }

    #[inline(always)]
    pub fn version(&self) -> Version {
        self.header.source_version()
    }
}

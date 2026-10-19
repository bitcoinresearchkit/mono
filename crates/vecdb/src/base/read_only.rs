use std::{marker::PhantomData, sync::Arc};

use rawdb::Region;

use crate::{VecIndex, VecValue, Version};

use super::{Header, SharedLen};

/// The persisted state a stored vector's readers share: no pushed values, no rollback state.
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
    pub fn stored_len(&self) -> usize {
        self.stored_len.get()
    }

    #[inline(always)]
    pub fn version(&self) -> Version {
        self.header.source_version()
    }
}

use std::{marker::PhantomData, sync::Arc};

use log::debug;

use super::{super::CompressionStrategy, ReadOnlyCompressedVec, ReadWriteCompressedVec};
use crate::{
    Error, ImportOptions, ImportableVec, ReadableBoxedVec, ReadableCloneableVec, Result, StoredVec,
    VecIndex, VecValue, cache::CachePolicy, remove_vec_with,
};

impl<I, T, S, C: CachePolicy> ImportableVec for ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    /// Resets the stored data on a schema mismatch or a corrupt layout.
    fn import_with(options: ImportOptions) -> Result<Self> {
        match Self::import_strict_with(options) {
            Err(error)
                if error.is_schema_mismatch() || matches!(error, Error::CorruptedRegion { .. }) =>
            {
                debug!("Resetting {}...", options.name);
                remove_vec_with::<I>(options.db, options.name)?;
                options
                    .db
                    .remove_region_if_exists(&Self::pages_region_name_with(options.name))?;
                Self::import_strict_with(options)
            }
            res => res,
        }
    }
}

impl<I, T, S, C: CachePolicy> StoredVec for ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    type ReadOnly = ReadOnlyCompressedVec<I, T, S, C>;

    #[inline]
    fn read_only_clone(&self) -> Self::ReadOnly {
        ReadOnlyCompressedVec {
            base: self.base.read_only.clone(),
            cache: self.cache.clone(),
            pages: Arc::clone(&self.pages),
            _strategy: PhantomData,
        }
    }
}

impl<I, T, S, C: CachePolicy> ReadableCloneableVec<I, T> for ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    #[inline]
    fn read_only_boxed_clone(&self) -> ReadableBoxedVec<I, T> {
        ReadableBoxedVec::from_read_only(StoredVec::read_only_clone(self))
    }
}

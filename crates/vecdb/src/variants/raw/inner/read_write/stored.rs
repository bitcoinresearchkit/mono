use std::marker::PhantomData;

use log::debug;

use super::{super::RawStrategy, ReadOnlyRawVec, ReadWriteRawVec};
use crate::{
    ImportOptions, ImportableVec, ReadableBoxedVec, ReadableCloneableVec, Result, StoredVec,
    VecIndex, VecValue, cache::CachePolicy, remove_vec_with,
};

impl<I, T, S, C: CachePolicy> ImportableVec for ReadWriteRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    fn import_with(options: ImportOptions) -> Result<Self> {
        match Self::import_strict_with(options) {
            Err(error) if error.is_schema_mismatch() => {
                debug!("Resetting {}...", options.name);
                remove_vec_with::<I>(options.db, options.name)?;
                Self::import_strict_with(options)
            }
            res => res,
        }
    }
}

impl<I, T, S, C: CachePolicy> StoredVec for ReadWriteRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    type ReadOnly = ReadOnlyRawVec<I, T, S, C>;

    #[inline]
    fn read_only_clone(&self) -> Self::ReadOnly {
        ReadOnlyRawVec {
            base: self.base.read_only.clone(),
            cache: self.cache.clone(),
            _strategy: PhantomData,
        }
    }
}

impl<I, T, S, C: CachePolicy> ReadableCloneableVec<I, T> for ReadWriteRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    #[inline]
    fn read_only_boxed_clone(&self) -> ReadableBoxedVec<I, T> {
        ReadableBoxedVec::from_read_only(StoredVec::read_only_clone(self))
    }
}

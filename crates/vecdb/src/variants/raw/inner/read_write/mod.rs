use std::marker::PhantomData;

use super::{RawStrategy, ReadOnlyRawVec, persisted::RawPersisted};
use crate::{
    AnyStoredVec, Error, HEADER_OFFSET, ImportOptions, RawRangeCursor, ReadWriteBaseVec, Result,
    VecIndex, VecReader, VecValue, Version,
    cache::{CachePolicy, NoCache},
    variants::stored_reads::PersistedReads,
};

pub mod any_vec;
mod mutable;
pub mod readable;
mod stored;
pub mod typed;
pub mod writable;
mod write;

const VERSION: Version = Version::ONE;

/// Append-only raw vector over serialization strategy `S` ([`BytesVec`](crate::BytesVec),
/// `ZeroCopyVec`). [`MutableVec`](crate::MutableVec) wraps one to replace or delete values.
#[derive(Debug)]
#[must_use]
pub struct ReadWriteRawVec<I, T: VecValue, S, C: CachePolicy = NoCache> {
    base: ReadWriteBaseVec<I, T>,
    cache: C::State<T>,
    _strategy: PhantomData<S>,
}

impl<I, T, S, C: CachePolicy> ReadWriteRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    const SIZE_OF_T: usize = size_of::<T>();

    /// Imports without resetting: a version or format mismatch is an error.
    fn import_strict_with(mut options: ImportOptions) -> Result<Self> {
        options.version = options.version + VERSION;

        let name = options.name;

        let cache = C::create()?;
        let base = ReadWriteBaseVec::import(options, S::FORMAT)?;

        // Raw format requires data to be aligned to SIZE_OF_T
        let region_len = base.region.meta().byte_len();
        if region_len > HEADER_OFFSET
            && !(region_len - HEADER_OFFSET).is_multiple_of(Self::SIZE_OF_T)
        {
            return Err(Error::CorruptedRegion {
                name: name.to_string(),
                region_len,
            });
        }

        let mut this = Self {
            base,
            cache,
            _strategy: PhantomData,
        };

        let len = this.real_stored_len();
        this.base.previous_stored_len = len;
        this.base.stored_len.set(len);

        Ok(this)
    }

    fn remove(self) -> Result<()> {
        self.base.remove()
    }

    #[inline]
    pub fn reader(&self) -> VecReader<I, T, S> {
        VecReader::from_read_write(self)
    }

    /// Creates a forward cursor over a bounded persisted range.
    #[inline]
    pub fn range_cursor_at(&self, from: usize, to: usize) -> RawRangeCursor<'_, T, S> {
        RawRangeCursor::new(self.region(), self.stored_len(), from, to)
    }

    /// Reads from the persisted or pushed layer.
    #[inline(always)]
    pub fn get_append_only(&self, index: I, reader: &VecReader<I, T, S>) -> Option<T> {
        let index = index.to_usize();
        // The reader snapshots the persisted boundary when it is created.
        // Using that boundary avoids reloading SharedLen for every lookup and
        // lets the inlined reader.get_at() reuse the same bounds check.
        let stored_len = reader.stored_len();
        debug_assert_eq!(stored_len, self.stored_len(), "stale VecReader");
        if index >= stored_len {
            return self.base.pushed().get(index - stored_len).cloned();
        }
        Some(reader.get_at(index))
    }

    /// The persisted rows `[from, to)`, past the stored length too (a pending truncation).
    fn collect_stored_range(&self, from: usize, to: usize) -> Result<Vec<T>> {
        let mut values = Vec::with_capacity(to - from);
        RawPersisted::<T, S>::new(self.region()).read(to, from, to, &mut values);
        debug_assert_eq!(values.len(), to - from, "rows missing on disk");
        Ok(values)
    }
}

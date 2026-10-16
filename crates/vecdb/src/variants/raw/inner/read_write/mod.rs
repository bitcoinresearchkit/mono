use std::marker::PhantomData;

use super::{RawStrategy, ReadOnlyRawVec, persisted::RawPersisted};
use crate::{
    AnyStoredVec, Error, HEADER_OFFSET, ImportOptions, RawRangeCursor, ReadWriteBaseVec,
    Result as CrateResult, VecIndex, VecReader, VecValue, Version,
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

/// Core implementation for raw storage vectors shared by BytesVec and ZeroCopyVec.
///
/// Parameterized by serialization strategy `S` to support different serialization approaches:
/// - `BytesStrategy`: Explicit little-endian serialization (portable)
/// - `ZeroCopyStrategy`: Native byte order via zerocopy (fast but not portable)
///
/// This is deliberately append-only. Existing-value mutation belongs to
/// [`MutableVec`](crate::MutableVec), which wraps a raw vector when needed.
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct ReadWriteRawVec<I, T: VecValue, S, C: CachePolicy = NoCache> {
    base: ReadWriteBaseVec<I, T>,
    pub(super) cache: C::State<T>,
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
    fn import_strict_with(mut options: ImportOptions) -> CrateResult<Self> {
        options.version = options.version + VERSION;

        let name = options.name;

        let cache = C::create()?;
        let base = ReadWriteBaseVec::import(options, S::FORMAT)?;

        // Raw format requires data to be aligned to SIZE_OF_T
        let region_len = base.region().meta().byte_len();
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
        *this.base.mut_prev_stored_len() = len;
        this.base.update_stored_len(len);

        Ok(this)
    }

    fn remove(self) -> CrateResult<()> {
        self.base.remove()
    }

    #[inline]
    pub fn reader(&self) -> VecReader<I, T, S> {
        VecReader::from_read_write(self)
    }

    /// Creates a forward cursor over a bounded persisted range.
    #[inline]
    pub fn range_cursor_at(&self, from: usize, to: usize) -> RawRangeCursor<'_, I, T, S> {
        RawRangeCursor::new(self.region(), self.stored_len(), from, to)
    }

    #[inline]
    fn index_to_name(&self) -> String {
        self.base.index_to_name()
    }

    /// Reads from the persisted or pushed layer.
    #[inline(always)]
    pub fn get_append_only(&self, index: I, reader: &VecReader<I, T, S>) -> Option<T> {
        self.get_append_only_at(index.to_usize(), reader)
    }

    /// Raw-index form of [`Self::get_append_only`].
    #[inline(always)]
    fn get_append_only_at(&self, index: usize, reader: &VecReader<I, T, S>) -> Option<T> {
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
    fn collect_stored_range(&self, from: usize, to: usize) -> CrateResult<Vec<T>> {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        RawPersisted::<I, T, S>::new(self.region()).read(to, from, to, &mut values);
        debug_assert_eq!(
            values.len(),
            to.saturating_sub(from),
            "rows missing on disk"
        );
        Ok(values)
    }
}

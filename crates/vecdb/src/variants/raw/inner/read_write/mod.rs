use std::{marker::PhantomData, result::Result};

use log::debug;
use rawdb::{Reader, Region};

use super::{RawStrategy, ReadOnlyRawVec};
use crate::{
    AnyStoredVec, Error, Format, HEADER_OFFSET, ImportOptions, RawIoSource, RawMmapSource,
    RawRangeCursor, ReadWriteBaseVec, Result as CrateResult, VecIndex, VecReader, VecValue,
    Version,
    cache::{CachePolicy, NoCache},
    vec_region_name_with,
};

pub mod any_stored_vec;
pub mod any_vec;
pub mod readable;
pub mod typed;
pub mod writable;

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

    pub(crate) fn read_only_clone(&self) -> ReadOnlyRawVec<I, T, S, C> {
        ReadOnlyRawVec {
            base: self.base.read_only_base(),
            cache: self.cache.clone(),
            _strategy: PhantomData,
        }
    }

    /// # Warning
    ///
    /// This will DELETE all existing data on format/version errors. Use with caution.
    pub(crate) fn forced_import_with(options: ImportOptions, format: Format) -> CrateResult<Self> {
        let res = Self::import_with(options, format);
        match res {
            Err(Error::WrongEndian)
            | Err(Error::WrongLength { .. })
            | Err(Error::DifferentFormat { .. })
            | Err(Error::DifferentVersion { .. }) => {
                debug!("Resetting {}...", options.name);
                options
                    .db
                    .remove_region_if_exists(&vec_region_name_with::<I>(options.name))?;
                Self::import_with(options, format)
            }
            _ => res,
        }
    }

    pub(crate) fn import_with(mut options: ImportOptions, format: Format) -> CrateResult<Self> {
        options.version = options.version + VERSION;

        let name = options.name;

        let cache = C::create()?;
        let base = ReadWriteBaseVec::import(options, format)?;

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

    pub(crate) fn remove(self) -> CrateResult<()> {
        self.base.remove()
    }

    #[inline(always)]
    pub(crate) fn mut_pushed(&mut self) -> &mut Vec<T> {
        self.base.mut_pushed()
    }

    #[inline]
    pub(crate) fn reserve_pushed(&mut self, additional: usize) {
        self.base.reserve_pushed(additional);
    }

    #[inline]
    fn raw_reader(&self) -> Reader {
        self.base.region().create_reader()
    }

    #[inline]
    pub(crate) fn reader(&self) -> VecReader<I, T, S> {
        VecReader::from_read_write(self)
    }

    /// Creates a forward cursor over a bounded persisted range.
    #[inline]
    pub fn range_cursor_at(&self, from: usize, to: usize) -> RawRangeCursor<'_, I, T, S> {
        RawRangeCursor::new(self.region(), self.stored_len(), from, to)
    }

    #[inline]
    pub(super) fn read_stored_into(
        region: &Region,
        len: usize,
        from: usize,
        to: usize,
        output: &mut Vec<T>,
    ) {
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return;
        }
        let offset = HEADER_OFFSET + from * Self::SIZE_OF_T;
        let bytes = (to - from) * Self::SIZE_OF_T;
        if region.prefers_mmap(offset, bytes) {
            RawMmapSource::<I, T, S>::new_from_parts(region, len, from, to).read_into(output);
        } else {
            RawIoSource::<I, T, S>::new_from_parts(region, len, from, to).read_into(output);
        }
    }

    #[inline]
    fn index_to_name(&self) -> String {
        self.base.index_to_name()
    }

    #[inline(always)]
    fn unchecked_read_at(&self, index: usize, reader: &Reader) -> T {
        let ptr = reader.read_from(HEADER_OFFSET).as_ptr();
        unsafe { S::read_from_ptr(ptr, index * Self::SIZE_OF_T) }
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

    pub(crate) fn base(&self) -> &ReadWriteBaseVec<I, T> {
        &self.base
    }
    pub(crate) fn base_mut(&mut self) -> &mut ReadWriteBaseVec<I, T> {
        &mut self.base
    }
    fn collect_stored_range(&self, from: usize, to: usize) -> CrateResult<Vec<T>> {
        let reader = self.raw_reader();
        Ok((from..to)
            .map(|index| self.unchecked_read_at(index, &reader))
            .collect())
    }
    #[inline(always)]
    fn fold_source<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, f: F) -> B {
        let offset = HEADER_OFFSET + from * Self::SIZE_OF_T;
        let bytes = (to - from) * Self::SIZE_OF_T;
        if self.region().prefers_mmap(offset, bytes) {
            RawMmapSource::new(self, from, to).fold(init, f)
        } else {
            RawIoSource::new(self, from, to).fold(init, f)
        }
    }
    #[inline(always)]
    fn try_fold_source<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        let offset = HEADER_OFFSET + from * Self::SIZE_OF_T;
        let bytes = (to - from) * Self::SIZE_OF_T;
        if self.region().prefers_mmap(offset, bytes) {
            RawMmapSource::new(self, from, to).try_fold(init, f)
        } else {
            RawIoSource::new(self, from, to).try_fold(init, f)
        }
    }
}

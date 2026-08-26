use std::marker::PhantomData;

use log::info;
use rawdb::Reader;

mod any_stored_vec;
mod any_vec;
mod readable;
mod rollback;
mod typed;
mod writable;

use crate::{
    AnyStoredVec, AnyVec, Error, Format, HEADER_OFFSET, ImportOptions, RawIoSource, RawMmapSource,
    RawRangeCursor, ReadWriteBaseVec, VecIndex, VecReader, VecValue, Version, vec_region_name_with,
};

use super::{RawStrategy, ReadOnlyRawVec};

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
pub struct ReadWriteRawVec<I, T, S> {
    pub(crate) base: ReadWriteBaseVec<I, T>,
    _strategy: PhantomData<S>,
}

impl<I, T, S> ReadWriteRawVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    pub const SIZE_OF_T: usize = size_of::<T>();

    pub fn read_only_clone(&self) -> ReadOnlyRawVec<I, T, S> {
        ReadOnlyRawVec::new(self.base.read_only_base())
    }

    /// # Warning
    ///
    /// This will DELETE all existing data on format/version errors. Use with caution.
    pub fn forced_import_with(options: ImportOptions, format: Format) -> crate::Result<Self> {
        let res = Self::import_with(options, format);
        match res {
            Err(Error::WrongEndian)
            | Err(Error::WrongLength { .. })
            | Err(Error::DifferentFormat { .. })
            | Err(Error::DifferentVersion { .. }) => {
                info!("Resetting {}...", options.name);
                options
                    .db
                    .remove_region_if_exists(&vec_region_name_with::<I>(options.name))?;
                Self::import_with(options, format)
            }
            _ => res,
        }
    }

    pub fn import_with(mut options: ImportOptions, format: Format) -> crate::Result<Self> {
        options.version = options.version + VERSION;

        let name = options.name;

        let base = ReadWriteBaseVec::import(options, format)?;

        // Raw format requires data to be aligned to SIZE_OF_T
        let region_len = base.region().meta().len();
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
            _strategy: PhantomData,
        };

        let len = this.real_stored_len();
        *this.base.mut_prev_stored_len() = len;
        this.base.update_stored_len(len);

        Ok(this)
    }

    pub fn remove(self) -> crate::Result<()> {
        self.base.remove()
    }

    #[inline(always)]
    pub fn pushed(&self) -> &[T] {
        self.base.pushed()
    }

    #[inline(always)]
    pub fn mut_pushed(&mut self) -> &mut Vec<T> {
        self.base.mut_pushed()
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.base.mut_pushed().push(value);
    }

    #[inline]
    pub fn reserve_pushed(&mut self, additional: usize) {
        self.base.reserve_pushed(additional);
    }

    #[inline]
    fn raw_reader(&self) -> Reader {
        self.base.region().create_reader()
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
    pub fn index_to_name(&self) -> String {
        self.base.index_to_name()
    }

    #[inline(always)]
    fn unchecked_read_at(&self, index: usize, reader: &Reader) -> T {
        let ptr = reader.prefixed(HEADER_OFFSET).as_ptr();
        unsafe { S::read_from_ptr(ptr, index * Self::SIZE_OF_T) }
    }

    #[inline]
    pub fn read_at_once(&self, index: usize) -> crate::Result<T> {
        let len = self.stored_len();
        if index >= len {
            return Err(Error::IndexTooHigh {
                index,
                len,
                name: self.name().to_string(),
            });
        }

        Ok(self.base.region().with_read_bytes(|bytes| unsafe {
            S::read_from_ptr(bytes.as_ptr().add(HEADER_OFFSET), index * Self::SIZE_OF_T)
        }))
    }

    #[inline]
    pub fn read_once(&self, index: I) -> crate::Result<T> {
        self.read_at_once(index.to_usize())
    }

    /// Reads from the persisted or pushed layer.
    #[inline(always)]
    pub fn get_append_only(&self, index: I, reader: &VecReader<I, T, S>) -> Option<T> {
        self.get_append_only_at(index.to_usize(), reader)
    }

    /// Raw-index form of [`Self::get_append_only`].
    #[inline(always)]
    pub fn get_append_only_at(&self, index: usize, reader: &VecReader<I, T, S>) -> Option<T> {
        // The reader snapshots the persisted boundary when it is created.
        // Using that boundary avoids reloading SharedLen for every lookup and
        // lets the inlined reader.get_at() reuse the same bounds check.
        let stored_len = reader.len();
        debug_assert_eq!(stored_len, self.stored_len(), "stale VecReader");
        if index >= stored_len {
            return self.base.pushed().get(index - stored_len).cloned();
        }
        Some(reader.get_at(index))
    }

    pub(crate) fn collect_stored_range(&self, from: usize, to: usize) -> crate::Result<Vec<T>> {
        let reader = self.raw_reader();
        Ok((from..to)
            .map(|index| self.unchecked_read_at(index, &reader))
            .collect())
    }

    #[inline(always)]
    pub(super) fn fold_source<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        let offset = HEADER_OFFSET + from * Self::SIZE_OF_T;
        let bytes = (to - from) * Self::SIZE_OF_T;
        if self.region().prefers_mmap(offset, bytes) {
            RawMmapSource::new(self, from, to).fold(init, f)
        } else {
            RawIoSource::new(self, from, to).fold(init, f)
        }
    }

    #[inline(always)]
    pub(super) fn try_fold_source<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E> {
        let offset = HEADER_OFFSET + from * Self::SIZE_OF_T;
        let bytes = (to - from) * Self::SIZE_OF_T;
        if self.region().prefers_mmap(offset, bytes) {
            RawMmapSource::new(self, from, to).try_fold(init, f)
        } else {
            RawIoSource::new(self, from, to).try_fold(init, f)
        }
    }

    pub fn fold_stored_io<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        let stored_len = self.stored_len();
        let from = from.min(stored_len);
        let to = to.min(stored_len);
        if from >= to {
            return init;
        }
        RawIoSource::new(self, from, to).fold(init, f)
    }

    pub fn fold_stored_mmap<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        let stored_len = self.stored_len();
        let from = from.min(stored_len);
        let to = to.min(stored_len);
        if from >= to {
            return init;
        }
        RawMmapSource::new(self, from, to).fold(init, f)
    }
}

use std::{marker::PhantomData, sync::Arc};

use log::info;
use parking_lot::RwLock;
use rawdb::{Reader, Region, likely, unlikely};

mod any_stored_vec;
mod any_vec;
mod readable;
mod rollback;
mod typed;
mod writable;

use crate::{
    AnyStoredVec, AnyVec, Error, Format, ImportOptions, ReadWriteBaseVec, Result, VecIndex,
    VecValue, Version, WritableVec, vec_region_name_with,
};

use super::{CompressionStrategy, Pages, ReadOnlyCompressedVec};

/// Maximum size in bytes of a single uncompressed page (16 KiB).
/// Smaller pages reduce memory overhead during decompression and improve
/// random access performance, while larger pages compress more efficiently.
/// 16 KiB balances these trade-offs for typical workloads.
pub const MAX_UNCOMPRESSED_PAGE_SIZE: usize = 16 * 1024;

const VERSION: Version = Version::new(3);

/// Inner implementation for compressed storage vectors.
/// Parameterized by compression strategy to support different compression algorithms.
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct ReadWriteCompressedVec<I, T, S> {
    pub(super) base: ReadWriteBaseVec<I, T>,
    pub(super) pages: Arc<RwLock<Pages>>,
    _strategy: PhantomData<S>,
}

impl<I, T, S> ReadWriteCompressedVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    pub(super) const PER_PAGE: usize = MAX_UNCOMPRESSED_PAGE_SIZE / Self::SIZE_OF_T;

    pub fn read_only_clone(&self) -> ReadOnlyCompressedVec<I, T, S> {
        ReadOnlyCompressedVec::new(self.base.read_only_base(), Arc::clone(&self.pages))
    }

    /// # Warning
    ///
    /// This will DELETE all existing data on format/version errors. Use with caution.
    pub fn forced_import_with(options: ImportOptions, format: Format) -> Result<Self> {
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
                options
                    .db
                    .remove_region_if_exists(&Self::pages_region_name_with(options.name))?;
                Self::import_with(options, format)
            }
            _ => res,
        }
    }

    #[inline]
    pub fn import_with(mut options: ImportOptions, format: Format) -> Result<Self> {
        options.version = options.version + VERSION;
        let db = options.db;
        let name = options.name;

        let base = ReadWriteBaseVec::import(options, format)?;

        let pages = Pages::import(db, &Self::pages_region_name_with(name))?;

        let mut this = Self {
            base,
            pages: Arc::new(RwLock::new(pages)),
            _strategy: PhantomData,
        };

        let len = this.real_stored_len();
        *this.base.mut_prev_stored_len() = len;
        this.base.update_stored_len(len);

        Ok(this)
    }

    #[inline]
    pub fn decode_page(&self, page_index: usize, reader: &Reader) -> Result<Vec<T>> {
        Self::decode_page_with(self.stored_len(), page_index, reader, &self.pages.read())
    }

    #[inline]
    pub(crate) fn decode_page_with(
        stored_len: usize,
        page_index: usize,
        reader: &Reader,
        pages: &Pages,
    ) -> Result<Vec<T>> {
        let index = Self::page_index_to_index(page_index);

        if unlikely(index >= stored_len) {
            return Err(Error::IndexTooHigh {
                index,
                len: stored_len,
                name: "page".to_string(),
            });
        }
        if unlikely(page_index >= pages.len()) {
            return Err(Error::ExpectVecToHaveIndex);
        }

        // SAFETY: We checked page_index < pages.len() above
        let page = pages
            .get(page_index)
            .expect("page should exist after bounds check");
        let data = reader.unchecked_read(page.start as usize, page.bytes as usize);
        S::decode_page(data, page)
    }

    #[inline]
    pub(super) fn compress_page(chunk: &[T]) -> Result<Vec<u8>> {
        debug_assert!(
            chunk.len() <= Self::PER_PAGE,
            "chunk length {} exceeds PER_PAGE {}",
            chunk.len(),
            Self::PER_PAGE
        );

        S::compress(chunk)
    }

    #[inline(always)]
    pub(crate) fn index_to_page_index(index: usize) -> usize {
        index / Self::PER_PAGE
    }

    #[inline(always)]
    pub(crate) fn page_index_to_index(page_index: usize) -> usize {
        page_index * Self::PER_PAGE
    }

    /// Reads stored page data into a buffer. Used by both ReadWrite and ReadOnly read_into_at.
    #[inline(always)]
    pub(crate) fn read_stored_pages_into(
        reader: &Reader,
        pages: &Pages,
        from: usize,
        to: usize,
        buf: &mut Vec<T>,
    ) {
        let start_page = Self::index_to_page_index(from);
        let end_page = Self::index_to_page_index(to - 1);
        for page_idx in start_page..=end_page {
            let page_start = Self::page_index_to_index(page_idx);
            let page = pages
                .get(page_idx)
                .expect("page should exist after bounds check");
            let data = reader.unchecked_read(page.start as usize, page.bytes as usize);
            let values_count = page.values_count() as usize;
            let local_from = from.saturating_sub(page_start);
            let local_to = (to - page_start).min(values_count);

            if !page.is_raw() && likely(local_from == 0) {
                let before = buf.len();
                S::decompress_append(data, values_count, buf)
                    .expect("decompression failed in read_into_at");
                buf.truncate(before + local_to);
            } else {
                let mut page_buf = Vec::with_capacity(values_count);
                S::decode_page_into(data, page, &mut page_buf)
                    .expect("page decode failed in read_into_at");
                buf.extend_from_slice(&page_buf[local_from..local_to]);
            }
        }
    }

    #[inline]
    pub(crate) fn prefers_mmap(
        region: &Region,
        pages: &RwLock<Pages>,
        from: usize,
        to: usize,
    ) -> bool {
        let Some((offset, len)) = pages.read().stored_byte_range(from, to, Self::PER_PAGE) else {
            return true;
        };
        region.prefers_mmap(offset, len)
    }

    pub(crate) fn pages_region_name(&self) -> String {
        Self::pages_region_name_with(self.name())
    }

    fn pages_region_name_with(name: &str) -> String {
        format!("{}_pages", vec_region_name_with::<I>(name))
    }

    pub fn remove(self) -> Result<()> {
        self.base.remove()?;

        let pages = Arc::try_unwrap(self.pages).map_err(|_| Error::PagesStillReferenced)?;
        pages.into_inner().remove()?;

        Ok(())
    }

    #[inline]
    pub fn reserve_pushed(&mut self, additional: usize) {
        self.base.reserve_pushed(additional);
    }

    #[inline]
    pub(crate) fn create_reader(&self) -> Reader {
        self.base.region().create_reader()
    }

    #[inline]
    pub(crate) fn pages(&self) -> &Arc<RwLock<Pages>> {
        &self.pages
    }

    pub(crate) fn collect_stored_range(&self, from: usize, to: usize) -> Result<Vec<T>> {
        if from >= to {
            return Ok(vec![]);
        }

        let reader = self.create_reader();
        let pages = self.pages.read();
        let real_len = pages.stored_len(Self::PER_PAGE);
        let to = to.min(real_len);
        if from >= to {
            return Ok(vec![]);
        }

        let mut result = Vec::with_capacity(to - from);
        let start_page = Self::index_to_page_index(from);
        let end_page = Self::index_to_page_index(to - 1);

        for page_idx in start_page..=end_page {
            let page_start = Self::page_index_to_index(page_idx);
            let decoded = Self::decode_page_with(real_len, page_idx, &reader, &pages)?;
            let local_from = from.saturating_sub(page_start);
            let local_to = (to - page_start).min(decoded.len());
            result.extend_from_slice(&decoded[local_from..local_to]);
        }

        Ok(result)
    }

    #[inline]
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
        crate::CompressedIoSource::new(self, from, to).fold(init, f)
    }

    #[inline]
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
        crate::CompressedMmapSource::new(self, from, to).fold(init, f)
    }

    #[inline(always)]
    pub(super) fn fold_source<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        let mmap = Self::prefers_mmap(self.region(), &self.pages, from, to);
        if mmap {
            crate::CompressedMmapSource::new(self, from, to).fold(init, f)
        } else {
            crate::CompressedIoSource::new(self, from, to).fold(init, f)
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
        let mmap = Self::prefers_mmap(self.region(), &self.pages, from, to);
        if mmap {
            crate::CompressedMmapSource::new(self, from, to).try_fold(init, f)
        } else {
            crate::CompressedIoSource::new(self, from, to).try_fold(init, f)
        }
    }
}

use std::{marker::PhantomData, result::Result, sync::Arc};

use log::debug;
use parking_lot::RwLock;
use rawdb::{Reader, Region};

use super::{CompressionStrategy, PAGES_PER_BLOCK, PageDecoder, Pages, ReadOnlyCompressedVec};
use crate::{
    AnyStoredVec, AnyVec, CompressedIoSource, CompressedMmapSource, CompressedRangeCursor, Error,
    Format, ImportOptions, ReadWriteBaseVec, Result as CrateResult, VecIndex, VecValue, Version,
    WritableVec,
    cache::{CachePolicy, NoCache},
    likely, unlikely, vec_region_name_with,
};

pub mod any_stored_vec;
pub mod any_vec;
pub mod readable;
pub mod typed;
pub mod writable;

/// Independently decodable uncompressed page size.
pub const COMPRESSED_PAGE_SIZE: usize = 8 * 1024;

const VERSION: Version = Version::new(4);

/// Inner implementation for compressed storage vectors.
/// Parameterized by compression strategy to support different compression algorithms.
#[derive(Debug)]
#[must_use = "Vector should be stored to keep data accessible"]
pub struct ReadWriteCompressedVec<I, T: VecValue, S, C: CachePolicy = NoCache> {
    base: ReadWriteBaseVec<I, T>,
    pub(super) cache: C::State<T>,
    pages: Arc<RwLock<Pages>>,
    pages_per_chunk: usize,
    _strategy: PhantomData<S>,
}

impl<I, T, S, C: CachePolicy> ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    pub fn read_only_clone(&self) -> ReadOnlyCompressedVec<I, T, S, C> {
        ReadOnlyCompressedVec {
            base: self.base.read_only_base(),
            cache: self.cache.clone(),
            pages: Arc::clone(&self.pages),
            _strategy: PhantomData,
        }
    }

    /// Creates a forward cursor over a bounded persisted range.
    #[inline]
    pub fn range_cursor_at(&self, from: usize, to: usize) -> CompressedRangeCursor<'_, I, T, S> {
        CompressedRangeCursor::new(self.region(), &self.pages, self.stored_len(), from, to)
    }

    /// # Warning
    ///
    /// This will DELETE all existing data on format/version errors. Use with caution.
    pub fn forced_import_with(options: ImportOptions, format: Format) -> CrateResult<Self> {
        let res = Self::import_with(options, format);
        match res {
            Err(Error::WrongEndian)
            | Err(Error::WrongLength { .. })
            | Err(Error::DifferentFormat { .. })
            | Err(Error::DifferentVersion { .. })
            | Err(Error::CorruptedRegion { .. }) => {
                debug!("Resetting {}...", options.name);
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
    pub fn import_with(mut options: ImportOptions, format: Format) -> CrateResult<Self> {
        options.version = options.version + VERSION;
        let db = options.db;
        let name = options.name;
        let initial_capacity = options.initial_capacity.unwrap_or(I::INITIAL_CAPACITY);
        let compression_chunk_size = options
            .max_compression_chunk_size
            .unwrap_or(S::MAX_UNCOMPRESSED_CHUNK_SIZE)
            .min(S::MAX_UNCOMPRESSED_CHUNK_SIZE);
        let pages_per_chunk =
            (compression_chunk_size / COMPRESSED_PAGE_SIZE).clamp(1, PAGES_PER_BLOCK);

        let cache = C::create()?;
        let base = ReadWriteBaseVec::import(options, format)?;

        let pages = Pages::import(
            db,
            &Self::pages_region_name_with(name),
            initial_capacity.div_ceil(Self::PER_PAGE),
        )?;
        let region_len = base.region().meta().byte_len();
        if pages.next_start() != u64::try_from(region_len).map_err(|_| Error::Overflow)? {
            return Err(Error::CorruptedRegion {
                name: name.to_string(),
                region_len,
            });
        }

        let mut this = Self {
            base,
            cache,
            pages: Arc::new(RwLock::new(pages)),
            pages_per_chunk,
            _strategy: PhantomData,
        };

        if Self::PER_PAGE == 0 {
            return Err(Error::InvalidArgument(
                "compressed values must fit in one page",
            ));
        }
        if this.pages.read().last().is_some_and(|page| {
            page.is_raw() && !(page.bytes as usize).is_multiple_of(Self::SIZE_OF_T)
        }) {
            return Err(Error::CorruptedRegion {
                name: name.to_string(),
                region_len: this.base.region().meta().byte_len(),
            });
        }
        let len = this.real_stored_len();
        *this.base.mut_prev_stored_len() = len;
        this.base.update_stored_len(len);

        Ok(this)
    }

    fn pages_region_name_with(name: &str) -> String {
        format!("{}_pages", vec_region_name_with::<I>(name))
    }

    pub fn remove(self) -> CrateResult<()> {
        self.base.remove()?;

        let pages = Arc::try_unwrap(self.pages).map_err(|_| Error::PagesStillReferenced)?;
        pages.into_inner().remove()?;

        Ok(())
    }

    #[inline]
    pub fn reserve_pushed(&mut self, additional: usize) {
        self.base.reserve_pushed(additional);
    }

    pub const PER_PAGE: usize = COMPRESSED_PAGE_SIZE / size_of::<T>();

    #[inline]
    pub fn decode_page_with(
        stored_len: usize,
        page_index: usize,
        reader: &Reader,
        pages: &Pages,
    ) -> CrateResult<Vec<T>> {
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
        let header = reader.read(page.header_start as usize, page.header_len());
        let body = reader.read(page.start as usize, page.bytes as usize);
        let expected_len = page.values_count(Self::PER_PAGE, Self::SIZE_OF_T);
        let mut values = Vec::with_capacity(expected_len);
        PageDecoder::<T, S>::default().decode_into(
            page,
            header,
            body,
            expected_len,
            &mut values,
        )?;
        Ok(values)
    }
    #[inline(always)]
    pub fn index_to_page_index(index: usize) -> usize {
        index / Self::PER_PAGE
    }
    #[inline(always)]
    pub fn page_index_to_index(page_index: usize) -> usize {
        page_index * Self::PER_PAGE
    }
    /// Reads stored page data into a buffer. Used by both ReadWrite and ReadOnly read_into_at.
    #[inline(always)]
    pub fn read_stored_pages_into(
        reader: &Reader,
        pages: &Pages,
        from: usize,
        to: usize,
        buf: &mut Vec<T>,
    ) {
        let start_page = Self::index_to_page_index(from);
        let end_page = Self::index_to_page_index(to - 1);
        let mut decoder = PageDecoder::<T, S>::default();
        let mut page_buf = Vec::with_capacity(Self::PER_PAGE);
        for page_idx in start_page..=end_page {
            let page_start = Self::page_index_to_index(page_idx);
            let page = pages
                .get(page_idx)
                .expect("page should exist after bounds check");
            let header = reader.read(page.header_start as usize, page.header_len());
            let body = reader.read(page.start as usize, page.bytes as usize);
            let values_count = page.values_count(Self::PER_PAGE, Self::SIZE_OF_T);
            let local_from = from.saturating_sub(page_start);
            let local_to = (to - page_start).min(values_count);

            if !page.is_raw() && likely(local_from == 0) {
                let before = buf.len();
                decoder
                    .decode_append(page, header, body, values_count, buf)
                    .expect("decompression failed in read_into_at");
                buf.truncate(before + local_to);
            } else {
                decoder
                    .decode_into(page, header, body, values_count, &mut page_buf)
                    .expect("page decode failed in read_into_at");
                buf.extend_from_slice(&page_buf[local_from..local_to]);
            }
        }
    }

    /// Batch adjacent requested pages. Separate disjoint runs before choosing
    /// mmap versus buffered I/O, so sparse reads do not prefetch unused gaps.
    pub(crate) fn read_sorted_stored_into(
        region: &Region,
        pages: &Arc<RwLock<Pages>>,
        stored_len: usize,
        mut indices: &[usize],
        out: &mut Vec<T>,
    ) {
        while let Some(&first) = indices.first() {
            let mut count = 1;
            let mut last_page = first / Self::PER_PAGE;
            while count < indices.len() {
                let page = indices[count] / Self::PER_PAGE;
                if page > last_page + 1 {
                    break;
                }
                last_page = page;
                count += 1;
            }
            let (run, rest) = indices.split_at(count);
            CompressedRangeCursor::<I, T, S>::new(
                region,
                pages,
                stored_len,
                first,
                run[count - 1] + 1,
            )
            .read_sorted_into(run, out);
            indices = rest;
        }
    }
    #[inline]
    pub fn prefers_mmap(region: &Region, pages: &RwLock<Pages>, from: usize, to: usize) -> bool {
        let Some((offset, len)) = pages.read().stored_byte_range(from, to, Self::PER_PAGE) else {
            return true;
        };
        region.prefers_mmap(offset, len)
    }
    fn pages_region_name(&self) -> String {
        Self::pages_region_name_with(self.name())
    }
    #[inline]
    fn create_reader(&self) -> Reader {
        self.base.region().create_reader()
    }
    #[inline]
    pub fn pages(&self) -> &Arc<RwLock<Pages>> {
        &self.pages
    }
    fn collect_stored_range(&self, from: usize, to: usize) -> CrateResult<Vec<T>> {
        if from >= to {
            return Ok(vec![]);
        }

        let reader = self.create_reader();
        let pages = self.pages.read();
        let real_len = pages.stored_len(Self::PER_PAGE, Self::SIZE_OF_T);
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
    #[inline(always)]
    fn fold_source<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, f: F) -> B {
        let mmap = Self::prefers_mmap(self.region(), &self.pages, from, to);
        if mmap {
            CompressedMmapSource::new(self, from, to).fold(init, f)
        } else {
            CompressedIoSource::new(self, from, to).fold(init, f)
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
        let mmap = Self::prefers_mmap(self.region(), &self.pages, from, to);
        if mmap {
            CompressedMmapSource::new(self, from, to).try_fold(init, f)
        } else {
            CompressedIoSource::new(self, from, to).try_fold(init, f)
        }
    }
}

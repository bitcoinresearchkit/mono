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
    remove_vec_with, vec_region_name_with,
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
    pub fn import_with(options: ImportOptions, format: Format) -> CrateResult<Self> {
        let res = Self::import_strict_with(options, format);
        match res {
            Err(Error::WrongLength { .. })
            | Err(Error::DifferentFormat { .. })
            | Err(Error::DifferentValueSize { .. })
            | Err(Error::DifferentVersion { .. })
            | Err(Error::CorruptedRegion { .. }) => {
                debug!("Resetting {}...", options.name);
                remove_vec_with::<I>(options.db, options.name)?;
                options
                    .db
                    .remove_region_if_exists(&Self::pages_region_name_with(options.name))?;
                Self::import_strict_with(options, format)
            }
            _ => res,
        }
    }

    #[inline]
    fn import_strict_with(mut options: ImportOptions, format: Format) -> CrateResult<Self> {
        const { assert!(Self::PER_PAGE > 0, "compressed values must fit in one page") };
        options.version = options.version + VERSION;
        let db = options.db;
        let name = options.name;
        let initial_capacity = options.initial_capacity.unwrap_or(I::INITIAL_CAPACITY);

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
            _strategy: PhantomData,
        };

        if this.pages.read_recursive().last().is_some_and(|page| {
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
    /// Pages compressed together: the strategy's chunk size, within one page block.
    const PAGES_PER_CHUNK: usize = {
        let pages = S::MAX_UNCOMPRESSED_CHUNK_SIZE / COMPRESSED_PAGE_SIZE;
        if pages == 0 {
            1
        } else if pages > PAGES_PER_BLOCK {
            PAGES_PER_BLOCK
        } else {
            pages
        }
    };

    #[inline(always)]
    pub fn index_to_page_index(index: usize) -> usize {
        index / Self::PER_PAGE
    }
    #[inline(always)]
    pub fn page_index_to_index(page_index: usize) -> usize {
        page_index * Self::PER_PAGE
    }
    /// Reads stored page data into a buffer through one decoder. Used by both ReadWrite and
    /// ReadOnly read_into_at, and by rewrites.
    #[inline(always)]
    pub fn read_stored_pages_into(
        reader: &Reader,
        pages: &Pages,
        from: usize,
        to: usize,
        buf: &mut Vec<T>,
    ) -> CrateResult<()> {
        // The page table is the published length: a truncation can land after a caller's length.
        let to = to.min(pages.stored_len(Self::PER_PAGE, Self::SIZE_OF_T));
        if from >= to {
            return Ok(());
        }
        let start_page = Self::index_to_page_index(from);
        let end_page = Self::index_to_page_index(to - 1);
        let mut decoder = PageDecoder::<T, S>::default();
        let mut scratch = Vec::new();
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
            decoder.decode_range_append(
                page,
                header,
                body,
                values_count,
                local_from..local_to,
                &mut scratch,
                buf,
            )?;
        }
        Ok(())
    }

    /// Visits stored values in `[from, to)` one decoded page at a time, through one decoder:
    /// mapped when resident, else buffered I/O.
    pub(crate) fn for_each_stored_chunk(
        region: &Region,
        pages: &Arc<RwLock<Pages>>,
        stored_len: usize,
        from: usize,
        to: usize,
        f: &mut dyn FnMut(usize, &[T]),
    ) {
        let to = to.min(stored_len);
        if from >= to {
            return;
        }
        if Self::prefers_mmap(region, pages, from, to) {
            CompressedMmapSource::<I, T, S>::new_from_parts(region, pages, stored_len, from, to)
                .for_each_chunk(f);
        } else {
            CompressedIoSource::<I, T, S>::new_from_parts(region, pages, stored_len, from, to)
                .for_each_chunk(f);
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
        let Some((offset, len)) =
            pages
                .read_recursive()
                .stored_byte_range(from, to, Self::PER_PAGE)
        else {
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
    /// Rewrites the region from `at` and publishes the matching page table and length with it.
    /// Readers take the pages before the region, so they pair old bytes with the old layout or
    /// new bytes with the new.
    fn publish(
        &mut self,
        at: usize,
        bytes: &[u8],
        stored_len: usize,
        update_pages: impl FnOnce(&mut Pages) -> CrateResult<()>,
    ) -> CrateResult<()> {
        let mut pages = self.pages.write();
        self.base.region().truncate_write(at, bytes)?;
        update_pages(&mut pages)?;
        pages.flush()?;
        self.base.update_stored_len(stored_len);
        drop(pages);
        self.base.write_header_if_needed()?;
        Ok(())
    }

    fn collect_stored_range(&self, from: usize, to: usize) -> CrateResult<Vec<T>> {
        // Pages before the region: the writer holds the pages while it rewrites the region.
        let pages = self.pages.read_recursive();
        let reader = self.create_reader();
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        Self::read_stored_pages_into(&reader, &pages, from, to, &mut values)?;
        Ok(values)
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

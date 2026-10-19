use std::{marker::PhantomData, sync::Arc};

use parking_lot::RwLock;
use rawdb::Region;

use super::{CompressionStrategy, PAGES_PER_BLOCK, Pages, ReadOnlyCompressedVec};
use crate::{
    AnyStoredVec, AnyVec, CompressedMmapSource, Error, ImportOptions, ReadWriteBaseVec, Result,
    VecIndex, VecValue, Version, WritableVec,
    cache::{CachePolicy, NoCache},
    vec_region_name_with,
};

pub mod any_vec;
pub mod readable;
mod stored;
pub mod typed;
pub mod writable;
mod write;

/// Independently decodable uncompressed page size.
pub const COMPRESSED_PAGE_SIZE: usize = 8 * 1024;

const VERSION: Version = Version::new(4);

/// Append-only paged vector over compression strategy `S`; the last, partial page stays raw.
#[derive(Debug)]
#[must_use]
pub struct ReadWriteCompressedVec<I, T: VecValue, S, C: CachePolicy = NoCache> {
    base: ReadWriteBaseVec<I, T>,
    cache: C::State<T>,
    pages: Arc<RwLock<Pages>>,
    _strategy: PhantomData<S>,
}

impl<I, T, S, C: CachePolicy> ReadWriteCompressedVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: CompressionStrategy<T>,
{
    /// Creates a forward cursor over a bounded persisted range.
    #[cfg(any(feature = "lz4", feature = "pco", feature = "zstd"))]
    #[inline]
    pub fn range_cursor_at(
        &self,
        from: usize,
        to: usize,
    ) -> crate::CompressedRangeCursor<'_, T, S> {
        crate::CompressedRangeCursor::new(self.region(), &self.pages, self.stored_len(), from, to)
    }

    fn import_strict_with(mut options: ImportOptions) -> Result<Self> {
        const { assert!(Self::PER_PAGE > 0, "compressed values must fit in one page") };
        options.version = options.version + VERSION;
        let db = options.db;
        let name = options.name;

        let cache = C::create()?;
        let base = ReadWriteBaseVec::import(options, S::FORMAT)?;

        let pages = Pages::import(
            db,
            &Self::pages_region_name_with(name),
            I::INITIAL_CAPACITY.div_ceil(Self::PER_PAGE),
        )?;
        let region_len = base.region.meta().byte_len();
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

        if this
            .pages
            .read_recursive()
            .last()
            .is_some_and(|page| page.raw && !(page.bytes as usize).is_multiple_of(Self::SIZE_OF_T))
        {
            return Err(Error::CorruptedRegion {
                name: name.to_string(),
                region_len: this.base.region.meta().byte_len(),
            });
        }
        let len = this.real_stored_len();
        this.base.previous_stored_len = len;
        this.base.stored_len.set(len);

        Ok(this)
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
    /// Rewrites the region from `at` and publishes the matching page table and length with it.
    /// Readers take the pages before the region, so they pair old bytes with the old layout or
    /// new bytes with the new.
    fn publish(
        &mut self,
        at: usize,
        bytes: &[u8],
        stored_len: usize,
        update_pages: impl FnOnce(&mut Pages) -> Result<()>,
    ) -> Result<()> {
        let mut pages = self.pages.write();
        self.base.region.truncate_write(at, bytes)?;
        update_pages(&mut pages)?;
        pages.flush()?;
        self.base.stored_len.set(stored_len);
        drop(pages);
        self.base.write_header_if_needed()?;
        Ok(())
    }

    /// The persisted rows `[from, to)`, past the stored length too (a pending truncation).
    fn collect_stored_range(&self, from: usize, to: usize) -> Result<Vec<T>> {
        let mut values = Vec::with_capacity(to - from);
        CompressedMmapSource::<T, S>::new_from_parts(self.region(), &self.pages, to, from, to)
            .read_into(&mut values)?;
        Ok(values)
    }
}

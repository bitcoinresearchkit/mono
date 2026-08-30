// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

pub mod block;
pub mod block_index;
mod bound;
pub mod data_block;
pub mod filter;
mod id;
mod index_block;
mod inner;
mod iter;
mod meta;
pub mod multi_writer;
mod owned_data_block_iter;
mod regions;
mod scanner;
pub mod util;
pub mod writer;

#[cfg(test)]
mod tests;

pub use block::{Block, BlockOffset};
pub use data_block::DataBlock;
pub use id::GlobalTableId;
pub use id::next_table_id;
pub use index_block::{BlockHandle, IndexBlock, KeyedBlockHandle};
pub use scanner::Scanner;

use crate::{
    CompressionType, Error, InternalValue, Result, Slice,
    cache::Cache,
    descriptor_table::DescriptorTable,
    file_accessor::FileAccessor,
    table::{
        block::{BlockType, ParsedItem},
        block_index::{BlockIndex, FullBlockIndex, TwoLevelBlockIndex, VolatileBlockIndex},
        filter::block::FilterBlock,
        meta::ParsedMeta,
        regions::ParsedRegions,
    },
    value::PointReadValue,
};
use block_index::BlockIndexImpl;
use bound::Bound as IterBound;
use byteorder::ReadBytesExt;
use inner::Inner;
use iter::Iter;
use std::{
    borrow::Cow,
    fmt,
    fs::File,
    ops::{Bound, Deref, RangeBounds},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use util::load_block;

/// A disk segment (a.k.a. `Table`, `SSTable`, `SST`, `sorted string table`) that is located on disk
///
/// A table is an immutable list of key-value pairs, split into compressed blocks.
/// A reference to the block (`block handle`) is saved in the "block index".
///
/// Deleted entries are represented by tombstones.
///
/// Tables can be merged together to improve read performance and free unneeded disk space by removing outdated item versions.
#[doc(alias("sstable", "sst", "sorted string table"))]
#[derive(Clone)]
pub struct Table(Arc<Inner>);

impl Deref for Table {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Table:{}({:?})", self.id(), self.metadata.key_range)
    }
}

impl Table {
    #[must_use]
    pub fn global_seqno(&self) -> u64 {
        self.0.global_seqno
    }

    /// Gets the global table ID.
    #[must_use]
    fn global_id(&self) -> GlobalTableId {
        (self.tree_id, self.id()).into()
    }

    #[must_use]
    pub fn filter_size(&self) -> u32 {
        self.regions.filter.map(|x| x.size()).unwrap_or_default()
    }

    #[must_use]
    pub fn pinned_filter_size(&self) -> usize {
        self.pinned_filter_block
            .as_ref()
            .map(FilterBlock::size)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn pinned_block_index_size(&self) -> usize {
        match &*self.block_index {
            BlockIndexImpl::Full(full_block_index) => full_block_index.inner().inner.size(),
            BlockIndexImpl::VolatileFull(_) => 0,
            BlockIndexImpl::TwoLevel(two_level_block_index) => {
                two_level_block_index.top_level_index.inner.size()
            }
        }
    }

    /// Gets the table ID.
    ///
    /// The table ID is unique for this tree, but not
    /// across multiple trees, use [`Table::global_id`] for that.
    #[must_use]
    pub fn id(&self) -> u32 {
        self.metadata.id
    }

    fn load_block(
        &self,
        handle: &BlockHandle,
        block_type: BlockType,
        compression: CompressionType,
    ) -> Result<Block> {
        load_block(
            self.global_id(),
            &self.path,
            &self.file_accessor,
            &self.cache,
            handle,
            block_type,
            compression,
        )
    }

    fn load_data_block(&self, handle: &BlockHandle) -> Result<DataBlock> {
        self.load_block(
            handle,
            BlockType::Data,
            self.metadata.data_block_compression,
        )
        .map(DataBlock::new)
    }

    /// Returns the (possibly compressed) file size.
    pub(crate) fn file_size(&self) -> u64 {
        self.metadata.file_size
    }

    pub fn get(&self, key: &[u8], key_hash: u64) -> Result<Option<InternalValue>> {
        let mut key_hash = Some(key_hash);
        self.get_with(key, &mut key_hash, Self::point_read)
    }

    pub fn get_value(
        &self,
        key: &[u8],
        key_hash: &mut Option<u64>,
    ) -> Result<Option<PointReadValue>> {
        self.get_with(key, key_hash, Self::point_read_value)
    }

    fn get_with<T>(
        &self,
        key: &[u8],
        key_hash: &mut Option<u64>,
        point_read: impl FnOnce(&Self, &[u8]) -> Result<Option<T>>,
    ) -> Result<Option<T>> {
        let filter_block = if let Some(block) = &self.pinned_filter_block {
            Some(Cow::Borrowed(block))
        } else if let Some(filter_idx) = &self.pinned_filter_index {
            let mut iter = filter_idx.iter();
            iter.seek(key, u64::MAX);

            if let Some(filter_block_handle) = iter.next() {
                let filter_block_handle = filter_block_handle.materialize(filter_idx.as_slice());

                let block = self.load_block(
                    &filter_block_handle.into_inner(),
                    BlockType::Filter,
                    CompressionType::None, // NOTE: We never write a filter block with compression
                )?;
                let block = FilterBlock::new(block);

                Some(Cow::Owned(block))
            } else {
                None
            }
        } else if let Some(filter_block_handle) = &self.regions.filter {
            let block = self.load_block(
                filter_block_handle,
                BlockType::Filter,
                CompressionType::None, // NOTE: We never write a filter block with compression
            )?;
            let block = FilterBlock::new(block);

            Some(Cow::Owned(block))
        } else {
            None
        };

        if let Some(filter_block) = &filter_block {
            let key_hash = *key_hash.get_or_insert_with(|| {
                crate::table::filter::standard_bloom::Builder::get_hash(key)
            });
            if !filter_block.maybe_contains_hash(key_hash)? {
                return Ok(None);
            }
        }

        point_read(self, key)
    }

    fn point_read(&self, key: &[u8]) -> Result<Option<InternalValue>> {
        self.point_read_with(key, |block, key| {
            block.point_read(key).map(|mut item| {
                item.key.seqno += self.global_seqno();
                item
            })
        })
    }

    fn point_read_value(&self, key: &[u8]) -> Result<Option<PointReadValue>> {
        self.point_read_with(key, DataBlock::point_read_value)
    }

    fn point_read_with<T>(
        &self,
        key: &[u8],
        point_read: impl Fn(&DataBlock, &[u8]) -> Option<T>,
    ) -> Result<Option<T>> {
        self.block_index.point_read(key, u64::MAX, |block_handle| {
            let block = self.load_data_block(block_handle)?;
            Ok(point_read(&block, key))
        })
    }

    /// Creates a scanner over the `Table`.
    ///
    /// The scanner is ĺogically the same as a normal iter(),
    /// however it uses its own file descriptor, does not look into the block cache
    /// and uses buffered I/O.
    ///
    /// Used for compactions and thus not available to a user.
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    #[doc(hidden)]
    pub fn scan(&self) -> Result<Scanner> {
        #[expect(
            clippy::expect_used,
            reason = "there shouldn't be 4 billion data blocks in a single table"
        )]
        let block_count = self
            .metadata
            .data_block_count
            .try_into()
            .expect("data block count should fit");

        Scanner::new(
            &self.path,
            block_count,
            self.metadata.data_block_compression,
            self.global_seqno(),
        )
    }

    /// Creates an iterator over the `Table`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    #[must_use]
    #[doc(hidden)]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = Result<InternalValue>> + use<> {
        self.range(..)
    }

    /// Creates a ranged iterator over the `Table`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    #[must_use]
    #[doc(hidden)]
    pub fn range<R: RangeBounds<Slice> + Send>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = Result<InternalValue>> + Send + use<R> {
        let index_iter = self.block_index.iter();

        let mut iter = Iter::new(self.clone(), index_iter);

        match range.start_bound() {
            Bound::Included(key) => iter.set_lower_bound(IterBound::Included(key.clone())),
            Bound::Excluded(key) => iter.set_lower_bound(IterBound::Excluded(key.clone())),
            Bound::Unbounded => {}
        }

        match range.end_bound() {
            Bound::Included(key) => iter.set_upper_bound(IterBound::Included(key.clone())),
            Bound::Excluded(key) => iter.set_upper_bound(IterBound::Excluded(key.clone())),
            Bound::Unbounded => {}
        }

        iter
    }

    fn read_tli(
        regions: &ParsedRegions,
        file: &File,
        compression: CompressionType,
    ) -> Result<IndexBlock> {
        log::trace!("Reading TLI block, with tli_ptr={:?}", regions.tli);

        let block = Block::from_file(file, regions.tli, compression)?;

        if block.header.block_type != BlockType::Index {
            return Err(Error::InvalidTag((
                "BlockType",
                block.header.block_type.into(),
            )));
        }

        Ok(IndexBlock::new(block))
    }

    /// Tries to recover a table from a file.
    #[expect(
        clippy::too_many_lines,
        reason = "table recovery mirrors the complete persisted table configuration"
    )]
    pub fn recover(
        file_path: PathBuf,
        global_seqno: u64,
        tree_id: u32,
        cache: Arc<Cache>,
        descriptor_table: Option<Arc<DescriptorTable>>,
        pin_filter: bool,
        pin_index: bool,
    ) -> Result<Self> {
        log::debug!("Recovering table from file {}", file_path.display());
        let mut file = File::open(&file_path)?;
        let file_path = Arc::new(file_path);

        let trailer = sfa::Reader::from_reader(&mut file)?;

        let table_version = trailer
            .toc()
            .section(b"table_version")
            .ok_or(Error::Unrecoverable)?;
        if table_version.len() != 1 {
            return Err(Error::Unrecoverable);
        }
        let version = ReadBytesExt::read_u8(&mut table_version.buf_reader(&file_path)?)?;
        if version != 8 {
            return Err(Error::InvalidVersion(version));
        }

        let regions = ParsedRegions::parse_from_toc(trailer.toc())?;

        log::trace!("Reading meta block, with meta_ptr={:?}", regions.metadata);
        let metadata = ParsedMeta::load_with_handle(&file, &regions.metadata)?;

        let file = Arc::new(file);

        let file_accessor = if let Some(dt) = descriptor_table {
            FileAccessor::DescriptorTable(dt)
        } else {
            FileAccessor::File(file.clone())
        };

        let block_index = if regions.index.is_some() {
            log::trace!(
                "Creating partitioned block index, with tli_ptr={:?}",
                regions.tli,
            );

            let block = Self::read_tli(&regions, &file, metadata.index_block_compression)?;

            BlockIndexImpl::TwoLevel(TwoLevelBlockIndex {
                top_level_index: block,
                cache: cache.clone(),
                compression: metadata.index_block_compression,
                path: Arc::clone(&file_path),
                file_accessor: file_accessor.clone(),
                table_id: (tree_id, metadata.id).into(),
            })
        } else if pin_index {
            log::trace!(
                "Creating pinned, full block index, with tli_ptr={:?}",
                regions.tli,
            );

            let block = Self::read_tli(&regions, &file, metadata.index_block_compression)?;
            BlockIndexImpl::Full(FullBlockIndex::new(block))
        } else {
            log::trace!("Creating volatile, full block index");

            BlockIndexImpl::VolatileFull(VolatileBlockIndex {
                cache: cache.clone(),
                compression: metadata.index_block_compression,
                file_accessor: file_accessor.clone(),
                handle: regions.tli,
                path: Arc::clone(&file_path),
                table_id: (tree_id, metadata.id).into(),
            })
        };

        let pinned_filter_index = if let Some(filter_tli_handle) = regions.filter_tli {
            let block =
                Block::from_file(&file, filter_tli_handle, metadata.index_block_compression)?;
            Some(IndexBlock::new(block))
        } else {
            None
        };

        // TODO: FilterBlock newtype
        let pinned_filter_block = if pinned_filter_index.is_none() && pin_filter {
            regions
                .filter
                .map(|filter_handle| {
                    log::debug!(
                        "Loading and pinning filter block, with filter_ptr={filter_handle:?}"
                    );

                    let block = Block::from_file(
                        &file,
                        filter_handle,
                        CompressionType::None, // NOTE: We never write a filter block with compression
                    )
                    .and_then(|block| {
                        if block.header.block_type == BlockType::Filter {
                            Ok(block)
                        } else {
                            Err(Error::InvalidTag((
                                "BlockType",
                                block.header.block_type.into(),
                            )))
                        }
                    })?;

                    Ok::<_, Error>(FilterBlock::new(block))
                })
                .transpose()?
        } else {
            None
        };

        log::debug!(
            "Recovered table #{} from {}",
            metadata.id,
            file_path.display(),
        );

        Ok(Self(Arc::new(Inner {
            path: file_path,
            tree_id,

            metadata,
            regions,

            cache,

            file_accessor,

            block_index: Arc::new(block_index),

            pinned_filter_index,

            pinned_filter_block,

            is_deleted: AtomicBool::default(),

            global_seqno,
        })))
    }

    pub(crate) fn mark_as_deleted(&self) {
        self.0.is_deleted.store(true, Ordering::Release);
    }

    /// Checks if a key range is (partially or fully) contained in this table.
    pub(crate) fn check_key_range_overlap(&self, bounds: &(Bound<&[u8]>, Bound<&[u8]>)) -> bool {
        self.metadata.key_range.overlaps_with_bounds(bounds)
    }

    /// Returns the highest sequence number in the table.
    #[must_use]
    pub fn get_highest_seqno(&self) -> u64 {
        self.metadata.highest_seqno + self.global_seqno()
    }
}

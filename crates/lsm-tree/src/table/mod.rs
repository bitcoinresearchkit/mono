// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    fmt,
    fs::File,
    ops::{Bound, Deref, RangeBounds},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use block_index::BlockIndexImpl;
use bound::Bound as IterBound;
use inner::Inner;
use log::{debug, trace};
use sfa::Reader;
use util::BlockSource;

use crate::{
    CompressionType, Config, Error, RecordBytes, Result, Slice,
    point_read_value::PointReadValue,
    table::{
        block::BlockType,
        block_index::{BlockIndex, FullBlockIndex, TwoLevelBlockIndex, VolatileBlockIndex},
        filter::standard_bloom::StandardBloomFilterReader,
        meta::ParsedMeta,
        regions::ParsedRegions,
    },
};

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

pub use block::{Block, BlockOffset};
pub use data_block::DataBlock;
pub use id::GlobalTableId;
pub use id::next_table_id;
pub use index_block::{BlockHandle, IndexBlock, KeyedBlockHandle};
pub use iter::Iter;
pub use scanner::Scanner;

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
        self.source.load(handle, block_type, compression)
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
    pub fn file_size(&self) -> u64 {
        self.metadata.file_size
    }

    pub fn get_value<V: RecordBytes>(
        &self,
        key: &[u8],
        key_hash: &mut Option<u64>,
    ) -> Result<Option<PointReadValue<V>>> {
        if !self.filter_may_contain(key, key_hash)? {
            return Ok(None);
        }
        self.block_index.point_read(key, u64::MAX, |handle| {
            self.load_data_block(handle)?.point_read_value(key)
        })
    }

    fn filter_may_contain(&self, key: &[u8], key_hash: &mut Option<u64>) -> Result<bool> {
        if let Some(block) = &self.pinned_filter_block {
            return Ok(block.maybe_contains(key, key_hash));
        }

        let handle = if let Some(index) = &self.pinned_filter_index {
            let mut iter = index.iter();
            iter.seek(key, u64::MAX);
            let Some(handle) = iter.next() else {
                return Ok(true);
            };
            BlockHandle::new(handle.offset, handle.size)
        } else if let Some(handle) = &self.regions.filter {
            *handle
        } else {
            return Ok(true);
        };

        let block = self.load_block(&handle, BlockType::Filter, CompressionType::None)?;
        Ok(StandardBloomFilterReader::new(block.data.as_ref())?.maybe_contains(key, key_hash))
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
    pub fn scan<K: RecordBytes, V: RecordBytes>(&self) -> Result<Scanner<K, V>> {
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
            &self.source.path,
            block_count,
            self.metadata.data_block_compression,
            self.global_seqno(),
        )
    }

    /// Iterates over the records in `range`, decoding them as `K` and `V`.
    pub fn range_as<K: RecordBytes, V: RecordBytes, R: RangeBounds<Slice>>(
        &self,
        range: R,
    ) -> Iter<K, V> {
        let index_iter = self.block_index.iter();

        let mut iter = Iter::<K, V>::new(self.clone(), index_iter);

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
        trace!("Reading TLI block, with tli_ptr={:?}", regions.tli);

        let block = Block::from_file(file, regions.tli, compression)?;

        if block.header.block_type != BlockType::Index {
            return Err(Error::InvalidTag((
                "BlockType",
                block.header.block_type.into(),
            )));
        }

        Ok(IndexBlock::new(block))
    }

    /// Opens a table file of `level`, pinning what the level's policies pin.
    pub fn recover(
        file_path: PathBuf,
        global_seqno: u64,
        tree_id: u32,
        level: usize,
        config: &Config,
    ) -> Result<Self> {
        debug!("Recovering table from file {}", file_path.display());
        let pin_filter = config.filter_block_pinning_policy.at_level(level);
        let pin_index = config.index_block_pinning_policy.at_level(level);
        let mut file = File::open(&file_path)?;
        let file_path = Arc::new(file_path);

        let trailer = Reader::from_reader(&mut file)?;

        let regions = ParsedRegions::parse_from_toc(trailer.toc())?;

        trace!("Reading meta block, with meta_ptr={:?}", regions.metadata);
        let metadata = ParsedMeta::load_with_handle(&file, &regions.metadata)?;

        let source = BlockSource {
            table_id: (tree_id, metadata.id).into(),
            path: file_path,
            descriptor_table: config.descriptor_table.clone(),
            cache: config.cache.clone(),
        };

        let block_index = if regions.index.is_some() {
            trace!(
                "Creating partitioned block index, with tli_ptr={:?}",
                regions.tli,
            );

            let block = Self::read_tli(&regions, &file, metadata.index_block_compression)?;

            BlockIndexImpl::TwoLevel(TwoLevelBlockIndex {
                top_level_index: block,
                source: source.clone(),
                compression: metadata.index_block_compression,
            })
        } else if pin_index {
            trace!(
                "Creating pinned, full block index, with tli_ptr={:?}",
                regions.tli,
            );

            let block = Self::read_tli(&regions, &file, metadata.index_block_compression)?;
            BlockIndexImpl::Full(FullBlockIndex::new(block))
        } else {
            trace!("Creating volatile, full block index");

            BlockIndexImpl::VolatileFull(VolatileBlockIndex {
                source: source.clone(),
                handle: regions.tli,
                compression: metadata.index_block_compression,
            })
        };

        let pinned_filter_index = if let Some(filter_tli_handle) = regions.filter_tli {
            let block =
                Block::from_file(&file, filter_tli_handle, metadata.index_block_compression)?;
            Some(IndexBlock::new(block))
        } else {
            None
        };

        let pinned_filter_block = if pinned_filter_index.is_none() && pin_filter {
            regions
                .filter
                .map(|filter_handle| {
                    debug!("Loading and pinning filter block, with filter_ptr={filter_handle:?}");

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

                    StandardBloomFilterReader::new(block.data)
                })
                .transpose()?
        } else {
            None
        };

        debug!(
            "Recovered table #{} from {}",
            metadata.id,
            source.path.display(),
        );

        Ok(Self(Arc::new(Inner {
            source,

            metadata,
            regions,

            block_index: Arc::new(block_index),

            pinned_filter_index,

            pinned_filter_block,

            is_deleted: AtomicBool::default(),

            global_seqno,
        })))
    }

    pub fn mark_as_deleted(&self) {
        self.0.is_deleted.store(true, Ordering::Release);
    }

    /// Returns the highest sequence number in the table.
    #[must_use]
    pub fn get_highest_seqno(&self) -> u64 {
        self.metadata.highest_seqno + self.global_seqno()
    }
}

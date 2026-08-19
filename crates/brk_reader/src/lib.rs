#![doc = include_str!("../README.md")]

use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Result as IoResult},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
    sync::Arc,
};

use brk_error::{Error, Result};
use brk_rpc::Client;
use brk_types::{BlkPosition, BlockHash, Height};
use parking_lot::RwLock;
use tracing::warn;

mod bisect;
mod blk_index_to_blk_path;
mod block_receiver;
mod canonical;
mod parse;
mod pipeline;
mod scan;
mod xor_bytes;
mod xor_index;

pub use blk_index_to_blk_path::BlkIndexToBlkPath;
pub use block_receiver::BlockReceiver;
pub use canonical::CanonicalRange;
pub use xor_bytes::*;
pub use xor_index::*;

/// bitcoind writes blocks slightly out of height order across files
/// during initial sync, headers-first body fetch, and reindex, so a
/// single "out of bounds" signal isn't enough to declare failure.
pub(crate) const OUT_OF_ORDER_FILE_BACKOFF: usize = 21;

const TARGET_NOFILE: u64 = 15_000;

/// Bitcoin Core blk-file reader. Cheap to clone (`Arc`-backed) and
/// thread-safe.
#[derive(Debug, Clone)]
pub struct Reader(Arc<ReaderInner>);

impl Reader {
    pub fn new(blocks_dir: PathBuf, client: &Client) -> Self {
        Self::raise_fd_limit();
        Self::new_without_rlimit(blocks_dir, client)
    }

    pub fn new_without_rlimit(blocks_dir: PathBuf, client: &Client) -> Self {
        Self(Arc::new(ReaderInner {
            xor_bytes: XORBytes::from(blocks_dir.as_path()),
            blk_file_cache: RwLock::new(BTreeMap::new()),
            blocks_dir,
            client: client.clone(),
        }))
    }

    /// Raises only the soft limit, clamped to the current hard limit:
    /// raising the hard limit requires `CAP_SYS_RESOURCE` and would
    /// fail on containers and unprivileged macOS processes.
    pub fn raise_fd_limit() {
        let (soft, hard) = rlimit::getrlimit(rlimit::Resource::NOFILE).unwrap_or((0, 0));
        let new_soft = soft.max(TARGET_NOFILE).min(hard);
        if new_soft > soft
            && let Err(e) = rlimit::setrlimit(rlimit::Resource::NOFILE, new_soft, hard)
        {
            warn!("failed to raise NOFILE rlimit: {e}");
        }
    }

    #[inline]
    pub fn client(&self) -> &Client {
        &self.0.client
    }

    #[inline]
    pub fn blocks_dir(&self) -> &Path {
        &self.0.blocks_dir
    }

    #[inline]
    pub fn xor_bytes(&self) -> XORBytes {
        self.0.xor_bytes
    }

    pub fn first_block_height(&self, blk_path: &Path, xor_bytes: XORBytes) -> Result<Height> {
        bisect::first_block_height(&self.0.client, blk_path, xor_bytes)
    }

    pub fn read_raw_bytes(&self, position: BlkPosition, size: usize) -> Result<Vec<u8>> {
        let file = self.0.open_blk(position.blk_index())?;
        let mut buffer = vec![0u8; size];
        file.read_exact_at(&mut buffer, position.offset() as u64)?;
        XORIndex::decode_at(&mut buffer, position.offset() as usize, self.0.xor_bytes);
        Ok(buffer)
    }

    pub fn reader_at(&self, position: BlkPosition) -> Result<BlkRead> {
        let file = self.0.open_blk(position.blk_index())?;
        Ok(BlkRead {
            file,
            offset: position.offset() as u64,
            xor_index: XORIndex::at_offset(position.offset() as usize),
            xor_bytes: self.0.xor_bytes,
        })
    }

    /// Streams every canonical block from genesis to the current
    /// chain tip.
    pub fn all(&self) -> Result<BlockReceiver> {
        self.after(None)
    }

    /// Streams every canonical block strictly after `hash` (or from
    /// genesis when `None`) up to the current chain tip.
    pub fn after(&self, hash: Option<BlockHash>) -> Result<BlockReceiver> {
        self.after_with(hash, pipeline::DEFAULT_PARSER_THREADS)
    }

    pub fn after_with(
        &self,
        hash: Option<BlockHash>,
        parser_threads: usize,
    ) -> Result<BlockReceiver> {
        let tip = self.0.client.get_last_height()?;
        let canonical = CanonicalRange::walk(&self.0.client, hash.as_ref(), tip)?;
        pipeline::spawn(self.0.clone(), canonical, parser_threads)
    }

    /// Inclusive height range `start..=end` in canonical order.
    pub fn range(&self, start: Height, end: Height) -> Result<BlockReceiver> {
        self.range_with(start, end, pipeline::DEFAULT_PARSER_THREADS)
    }

    pub fn range_with(
        &self,
        start: Height,
        end: Height,
        parser_threads: usize,
    ) -> Result<BlockReceiver> {
        let tip = self.0.client.get_last_height()?;
        if end > tip {
            return Err(Error::OutOfRange(
                format!("range end {end} is past current tip {tip}").into(),
            ));
        }
        let canonical = CanonicalRange::between(&self.0.client, start, end)?;
        pipeline::spawn(self.0.clone(), canonical, parser_threads)
    }
}

#[derive(Debug)]
pub(crate) struct ReaderInner {
    /// Invalidated on every `refresh_paths` so a pruned or reindexed
    /// blk file can't keep serving stale bytes from a dead inode.
    blk_file_cache: RwLock<BTreeMap<u16, Arc<File>>>,
    pub(crate) xor_bytes: XORBytes,
    pub(crate) blocks_dir: PathBuf,
    pub(crate) client: Client,
}

impl ReaderInner {
    pub(crate) fn refresh_paths(&self) -> Result<BlkIndexToBlkPath> {
        let paths = BlkIndexToBlkPath::scan(&self.blocks_dir)?;
        self.blk_file_cache.write().clear();
        Ok(paths)
    }

    fn open_blk(&self, blk_index: u16) -> Result<Arc<File>> {
        if let Some(file) = self.blk_file_cache.read().get(&blk_index).cloned() {
            return Ok(file);
        }
        let path = self.blocks_dir.join(format!("blk{blk_index:05}.dat"));
        let file = Arc::new(File::open(&path)?);
        let mut cache = self.blk_file_cache.write();
        Ok(cache.entry(blk_index).or_insert(file).clone())
    }
}

pub struct BlkRead {
    file: Arc<File>,
    offset: u64,
    xor_index: XORIndex,
    xor_bytes: XORBytes,
}

impl Read for BlkRead {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let n = self.file.read_at(buf, self.offset)?;
        self.xor_index.bytes(&mut buf[..n], self.xor_bytes);
        self.offset += n as u64;
        Ok(n)
    }
}

use std::{
    collections::BTreeMap,
    fs::{self, File, Metadata},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    sync::Arc,
};

use arc_swap::ArcSwap;
use brk_error::Result;
use brk_rpc::Client;
use brk_types::BlockHash;
use parking_lot::Mutex;

use crate::{BlkIndexToBlkPath, XORBytes};

// Bounds retained descriptors, not handles still owned by active readers.
const MAX_CACHED_FILES: usize = 64;

/// Where the last block streamed ends in its blk file: the next blocks are usually appended
/// right after it.
#[derive(Debug, Clone, Copy)]
pub struct AppendPosition {
    pub hash: BlockHash,
    pub blk_index: u16,
    /// The end of the block's frame: the next frame's start.
    pub end: usize,
    /// The file it was found in: a replaced pathname (a reindex rewriting files) holds other
    /// offsets.
    pub file: FileId,
}

/// A file's device and inode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileId(u64, u64);

impl From<&Metadata> for FileId {
    fn from(metadata: &Metadata) -> Self {
        Self(metadata.dev(), metadata.ino())
    }
}

#[derive(Debug)]
pub struct ReaderInner {
    blk_file_cache: ArcSwap<BTreeMap<u16, Arc<File>>>,
    append_position: Mutex<Option<AppendPosition>>,
    pub xor_bytes: XORBytes,
    pub blocks_dir: PathBuf,
    pub client: Client,
}

impl ReaderInner {
    /// A missing `xor.dat` is the identity mask; an unreadable or malformed one is an error.
    pub fn new(blocks_dir: PathBuf, client: &Client) -> Result<Self> {
        Ok(Self {
            xor_bytes: XORBytes::read(&blocks_dir)?.unwrap_or_default(),
            blk_file_cache: ArcSwap::default(),
            append_position: Mutex::default(),
            blocks_dir,
            client: client.clone(),
        })
    }

    pub fn append_position(&self) -> Option<AppendPosition> {
        *self.append_position.lock()
    }

    pub fn retain_append_position(&self, position: AppendPosition) {
        *self.append_position.lock() = Some(position);
    }

    pub fn refresh_paths(&self) -> Result<BlkIndexToBlkPath> {
        let paths = BlkIndexToBlkPath::scan(&self.blocks_dir);
        // A failed scan invalidates old cached inodes too. Active readers keep
        // their existing handle; subsequent opens resolve the current path.
        self.blk_file_cache.store(Arc::default());
        paths
    }

    fn blk_path(&self, blk_index: u16) -> PathBuf {
        self.blocks_dir.join(format!("blk{blk_index:05}.dat"))
    }

    /// The identity of the file the pathname of `blk_index` names now.
    pub fn blk_path_id(&self, blk_index: u16) -> Option<FileId> {
        fs::metadata(self.blk_path(blk_index))
            .ok()
            .map(|metadata| FileId::from(&metadata))
    }

    pub fn open_blk(&self, blk_index: u16) -> Result<Arc<File>> {
        let cache = self.blk_file_cache.load();
        if let Some(file) = cache.get(&blk_index) {
            return Ok(file.clone());
        }
        let file = Arc::new(File::open(self.blk_path(blk_index))?);
        let mut next = BTreeMap::clone(&cache);
        if next.len() == MAX_CACHED_FILES {
            // Favor newer block files without LRU work on hits.
            next.pop_first();
        }
        next.insert(blk_index, file.clone());
        // Only over the map this open started from: after a refresh (or a racing insert) the
        // handle stays uncached, so an open begun before a refresh never reinserts its old inode.
        self.blk_file_cache.compare_and_swap(&cache, Arc::new(next));
        Ok(file)
    }
}

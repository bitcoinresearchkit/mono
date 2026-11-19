use std::{collections::BTreeMap, fs::File, path::PathBuf, sync::Arc};

use arc_swap::ArcSwap;
use brk_error::Result;
use brk_rpc::Client;
use brk_types::BlockHash;
use parking_lot::Mutex;

use crate::XORBytes;

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

    /// Forgets the descriptors raw reads keep, for when the blk files may have been replaced.
    /// Active readers keep their existing handle; subsequent opens resolve the current path.
    pub fn close_blk_files(&self) {
        self.blk_file_cache.store(Arc::default());
    }

    pub fn blk_path(&self, blk_index: u16) -> PathBuf {
        self.blocks_dir.join(format!("blk{blk_index:05}.dat"))
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
        // Only over the map this open started from: after a close (or a racing insert) the
        // handle stays uncached, so an open begun before a close never reinserts its old inode.
        self.blk_file_cache.compare_and_swap(&cache, Arc::new(next));
        Ok(file)
    }
}

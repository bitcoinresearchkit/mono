use std::{collections::BTreeMap, fs::File, path::PathBuf, sync::Arc};

use arc_swap::ArcSwap;
use brk_error::Result;
use brk_rpc::Client;

use crate::{BlkIndexToBlkPath, XORBytes};

// Bounds retained descriptors, not handles still owned by active readers.
const MAX_CACHED_FILES: usize = 64;

#[derive(Debug)]
pub struct ReaderInner {
    blk_file_cache: ArcSwap<BTreeMap<u16, Arc<File>>>,
    pub xor_bytes: XORBytes,
    pub blocks_dir: PathBuf,
    pub client: Client,
}

impl ReaderInner {
    pub fn new(blocks_dir: PathBuf, client: &Client) -> Self {
        Self {
            xor_bytes: XORBytes::from(blocks_dir.as_path()),
            blk_file_cache: ArcSwap::default(),
            blocks_dir,
            client: client.clone(),
        }
    }

    pub fn refresh_paths(&self) -> Result<BlkIndexToBlkPath> {
        let paths = BlkIndexToBlkPath::scan(&self.blocks_dir);
        // A failed scan invalidates old cached inodes too. Active readers keep
        // their existing handle; subsequent opens resolve the current path.
        self.blk_file_cache.store(Arc::default());
        paths
    }

    pub fn open_blk(&self, blk_index: u16) -> Result<Arc<File>> {
        let cache = self.blk_file_cache.load();
        if let Some(file) = cache.get(&blk_index) {
            return Ok(file.clone());
        }
        let path = self.blocks_dir.join(format!("blk{blk_index:05}.dat"));
        let file = Arc::new(File::open(path)?);
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

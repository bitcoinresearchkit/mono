use std::{collections::BTreeMap, fs::File, path::PathBuf, sync::Arc};

use brk_error::Result;
use brk_rpc::Client;
use parking_lot::RwLock;

use crate::{BlkIndexToBlkPath, XORBytes};

// Bounds retained descriptors, not handles still owned by active readers.
const MAX_CACHED_FILES: usize = 64;

#[derive(Debug)]
pub struct ReaderInner {
    blk_file_cache: RwLock<BTreeMap<u16, Arc<File>>>,
    pub xor_bytes: XORBytes,
    pub blocks_dir: PathBuf,
    pub client: Client,
}

impl ReaderInner {
    pub fn new(blocks_dir: PathBuf, client: &Client) -> Self {
        Self {
            xor_bytes: XORBytes::from(blocks_dir.as_path()),
            blk_file_cache: RwLock::new(BTreeMap::new()),
            blocks_dir,
            client: client.clone(),
        }
    }

    pub fn refresh_paths(&self) -> Result<BlkIndexToBlkPath> {
        let paths = BlkIndexToBlkPath::scan(&self.blocks_dir);
        // A failed scan invalidates old cached inodes too. Active readers keep
        // their existing handle; subsequent opens resolve the current path.
        self.blk_file_cache.write().clear();
        paths
    }

    pub fn open_blk(&self, blk_index: u16) -> Result<Arc<File>> {
        if let Some(file) = self.blk_file_cache.read().get(&blk_index).cloned() {
            return Ok(file);
        }
        let path = self.blocks_dir.join(format!("blk{blk_index:05}.dat"));
        let mut cache = self.blk_file_cache.write();
        if let Some(file) = cache.get(&blk_index) {
            return Ok(file.clone());
        }
        // Serialize misses with invalidation: an open begun before a refresh
        // must not insert its old inode after that refresh cleared the cache.
        let file = Arc::new(File::open(path)?);
        if cache.len() == MAX_CACHED_FILES {
            // Favor newer block files without write-lock/LRU work on hits.
            cache.pop_first();
        }
        cache.insert(blk_index, file.clone());
        Ok(file)
    }
}

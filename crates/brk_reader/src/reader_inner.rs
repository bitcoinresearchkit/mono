use std::{fs::File, ops::Deref, path::PathBuf, sync::Arc};

use arc_swap::{ArcSwap, ArcSwapOption};
use bitcoin::Network;
use brk_error::Result;
use brk_rpc::Client;
use brk_types::{BlockHash, Chain};
use parking_lot::Mutex;

use crate::XORBytes;

/// Open blk files kept, direct-mapped by index (5,881 files on mainnet, 2026-11): a file stays
/// open until one a multiple of `BLK_SLOTS` away is read, and a miss replaces one slot. Bounds the
/// descriptors kept, not handles still owned by active readers.
const BLK_SLOTS: usize = 1024;

/// An open blk file and its index.
#[derive(Debug)]
pub struct BlkFile {
    index: u16,
    file: File,
}

impl Deref for BlkFile {
    type Target = File;

    fn deref(&self) -> &File {
        &self.file
    }
}

type BlkSlots = Box<[ArcSwapOption<BlkFile>]>;

fn empty_slots() -> Arc<BlkSlots> {
    Arc::new((0..BLK_SLOTS).map(|_| ArcSwapOption::empty()).collect())
}

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
    blk_files: ArcSwap<BlkSlots>,
    append_position: Mutex<Option<AppendPosition>>,
    pub chain: &'static Chain,
    pub magic: [u8; 4],
    pub xor_bytes: XORBytes,
    pub blocks_dir: PathBuf,
    pub client: Client,
}

impl ReaderInner {
    pub fn new(blocks_dir: PathBuf, client: &Client, network: Network) -> Result<Self> {
        Ok(Self {
            chain: Chain::for_network(network),
            magic: network.magic().to_bytes(),
            xor_bytes: XORBytes::read(&blocks_dir)?.unwrap_or_default(),
            blk_files: ArcSwap::new(empty_slots()),
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
        self.blk_files.store(empty_slots());
    }

    pub fn blk_path(&self, blk_index: u16) -> PathBuf {
        self.blocks_dir.join(format!("blk{blk_index:05}.dat"))
    }

    pub fn open_blk(&self, blk_index: u16) -> Result<Arc<BlkFile>> {
        let slots = self.blk_files.load();
        let slot = &slots[usize::from(blk_index) % BLK_SLOTS];
        let current = slot.load();
        if let Some(file) = current.as_ref().filter(|file| file.index == blk_index) {
            return Ok(file.clone());
        }
        let file = Arc::new(BlkFile {
            index: blk_index,
            file: File::open(self.blk_path(blk_index))?,
        });
        // Only over the slot this open saw: a racing open keeps its own handle. After a close the
        // slots are new and this lands in the dropped ones, so an open begun before a close never
        // keeps its old inode.
        slot.compare_and_swap(&current, Some(file.clone()));
        Ok(file)
    }
}

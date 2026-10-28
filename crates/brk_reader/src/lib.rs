#![doc = include_str!("../README.md")]

use std::{
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
    sync::Arc,
};

use bitcoin::Network;
use brk_error::{Error, Result};
use brk_rpc::Client;
use brk_types::{BlkPosition, BlockHash, Height, ReadBlock};
use canonical::CanonicalRange;
use crossbeam_channel::IntoIter;
use reader_inner::ReaderInner;
use xor_index::XORIndex;

mod bisect;
mod blk_index_to_blk_path;
mod blk_read;
mod canonical;
mod parse;
mod pipeline;
mod reader_inner;
mod scan;
mod xor_bytes;
mod xor_index;

pub(crate) use blk_index_to_blk_path::BlkIndexToBlkPath;
pub use blk_read::BlkRead;

pub use xor_bytes::*;

/// Stream of parsed blocks: each item waits for its block, and the stream ends when the pipeline
/// closes.
pub type BlockReceiver = IntoIter<Result<ReadBlock>>;

/// bitcoind writes blocks slightly out of height order across files
/// during initial sync, headers-first body fetch, and reindex, so a
/// single "out of bounds" signal isn't enough to declare failure.
const OUT_OF_ORDER_FILE_BACKOFF: usize = 21;

/// Bitcoin Core blk-file reader. Cheap to clone (`Arc`-backed) and
/// thread-safe.
#[derive(Debug, Clone)]
pub struct Reader(Arc<ReaderInner>);

impl Reader {
    /// Errs when the blocks directory's `xor.dat` is unreadable or malformed (a missing one is
    /// the identity mask).
    pub fn new(blocks_dir: PathBuf, client: &Client) -> Result<Self> {
        Ok(Self(Arc::new(ReaderInner::new(blocks_dir, client)?)))
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

    /// Errs unless the node runs mainnet: this reader parses mainnet's block files only (their
    /// magic), and another network's would read as no blocks at all.
    pub fn check_network(&self) -> Result<()> {
        match self.0.client.get_network()? {
            Network::Bitcoin => Ok(()),
            network => Err(Error::NotMainnet(network.to_string())),
        }
    }

    /// Errs when the blocks directory's `xor.dat` is unreadable or malformed, or no longer the
    /// mask this reader decodes with: another node's blocks are behind the path now.
    pub fn check_source(&self) -> Result<()> {
        if XORBytes::read(self.blocks_dir())?.unwrap_or_default() != self.xor_bytes() {
            return Err(Error::Internal(
                "Block source changed after the Reader was created",
            ));
        }
        Ok(())
    }

    pub fn read_raw_bytes(&self, position: BlkPosition, size: usize) -> Result<Vec<u8>> {
        let file = self.0.open_blk(position.blk_index())?;
        let mut buffer = vec![0u8; size];
        file.read_exact_at(&mut buffer, position.offset() as u64)?;
        XORIndex::at_offset(position.offset() as usize).bytes(&mut buffer, self.0.xor_bytes);
        Ok(buffer)
    }

    /// Streams every canonical block strictly after `hash` (or from
    /// genesis when `None`) up to the current chain tip.
    pub fn after(&self, hash: Option<BlockHash>) -> Result<BlockReceiver> {
        let tip = self.0.client.get_last_height()?;
        match CanonicalRange::walk(&self.0.client, hash.as_ref(), tip)? {
            Some(canonical) => pipeline::spawn(self.0.clone(), canonical),
            None => Ok(pipeline::failed(Error::Internal(
                "anchor is no longer on the best chain",
            ))),
        }
    }

    /// Inclusive height range `start..=end` in canonical order.
    pub fn range(&self, start: Height, end: Height) -> Result<BlockReceiver> {
        let tip = self.0.client.get_last_height()?;
        if end > tip {
            return Err(Error::OutOfRange(
                format!("range end {end} is past current tip {tip}").into(),
            ));
        }
        let canonical = CanonicalRange::between(&self.0.client, start, end)?;
        pipeline::spawn(self.0.clone(), canonical)
    }
}

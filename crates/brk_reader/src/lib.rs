#![doc = include_str!("../README.md")]

use std::{
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
    sync::Arc,
};

pub use bitcoin::Network;
use brk_error::{Error, Result};
use brk_rpc::{Client, ConnectArgs};
use brk_types::{BlkPosition, BlockHash, Chain, Height};
use canonical::CanonicalRange;
use reader_inner::ReaderInner;
use xor_index::XORIndex;

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
pub use pipeline::BlockReceiver;

pub use xor_bytes::*;

/// Bitcoin Core blk-file reader. Cheap to clone (`Arc`-backed) and
/// thread-safe.
#[derive(Debug, Clone)]
pub struct Reader(Arc<ReaderInner>);

impl Reader {
    /// Resolve the node's blocks directory using the same network read that sets its magic.
    pub fn connect(connect: &ConnectArgs, client: &Client) -> Result<Self> {
        let network = client.get_network()?;
        let blocks_dir = connect.blocks_dir(Chain::for_network(network))?;
        if !blocks_dir.is_dir() {
            return Err(Error::NotFound(format!(
                "{} isn't a valid blocks directory; set --blocksdir or --bitcoindir",
                blocks_dir.display()
            )));
        }
        Self::with_network(blocks_dir, client, network)
    }

    /// Constructs a reader with an already known network, including for offline imports.
    pub fn with_network(blocks_dir: PathBuf, client: &Client, network: Network) -> Result<Self> {
        let inner = ReaderInner::new(blocks_dir, client, network)?;
        inner.chain.activate()?;
        Ok(Self(Arc::new(inner)))
    }

    pub fn chain(&self) -> &'static Chain {
        self.0.chain
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
    /// genesis when `None`) up to the height `end`, the chain tip for a sync.
    pub fn after(&self, hash: Option<BlockHash>, end: Height) -> Result<BlockReceiver> {
        Ok(
            match CanonicalRange::walk(&self.0.client, hash.as_ref(), end)? {
                Some(canonical) => pipeline::spawn(self.0.clone(), canonical),
                None => pipeline::failed(Error::Internal("anchor is no longer on the best chain")),
            },
        )
    }
}

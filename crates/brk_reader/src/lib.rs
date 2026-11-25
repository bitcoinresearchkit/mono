#![doc = include_str!("../README.md")]

use std::{
    fs::File,
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

    /// Path of a Bitcoin Core block file.
    pub fn blk_path(&self, blk_index: u16) -> PathBuf {
        self.0.blk_path(blk_index)
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
        self.read_raw_in(&file, position, size)
    }

    /// The bytes at each of `places`, in order. Their ranges are advised together before any is
    /// read, so the device serves them as one batch rather than one read at a time, and each
    /// read uses the handle its advice opened. Ranges back to back in one file (a block's
    /// transactions) share one advice; a lone read gets none (it gains nothing from it).
    pub fn read_raw_batch(&self, places: &[(BlkPosition, usize)]) -> Result<Vec<Vec<u8>>> {
        let files = places
            .iter()
            .map(|&(position, _)| self.0.open_blk(position.blk_index()))
            .collect::<Result<Vec<_>>>()?;
        // (first place, start, end) of each run.
        let mut runs: Vec<(usize, u64, u64)> = Vec::new();
        for (k, &(position, size)) in places.iter().enumerate() {
            let start = u64::from(position.offset());
            let end = start + size as u64;
            match runs.last_mut() {
                Some((first, run_start, run_end))
                    if places[*first].0.blk_index() == position.blk_index()
                        && (*run_start..=*run_end).contains(&start) =>
                {
                    *run_end = (*run_end).max(end);
                }
                _ => runs.push((k, start, end)),
            }
        }
        if places.len() > 1 {
            for (first, start, end) in runs {
                rawdb::advise_read(&files[first], start, (end - start) as usize);
            }
        }
        places
            .iter()
            .zip(&files)
            .map(|(&(position, size), file)| self.read_raw_in(file, position, size))
            .collect()
    }

    fn read_raw_in(&self, file: &File, position: BlkPosition, size: usize) -> Result<Vec<u8>> {
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

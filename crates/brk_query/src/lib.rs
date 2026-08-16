#![doc = include_str!("../README.md")]
#![allow(clippy::module_inception)]

use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use brk_computer::Computer;
use brk_error::{OptionData, Result};
use brk_indexer::{Indexer, Lengths};
use brk_mempool::Mempool;
use brk_oracle::Oracle;
use brk_reader::Reader;
use brk_rpc::Client;
use brk_types::{BlockHash, BlockHashPrefix, Epoch, Halving, Height, Index, SyncStatus};
use vecdb::{ReadBounds, ReadOnlyClone, ReadableVec, Ro};

#[cfg(feature = "tokio")]
mod r#async;
mod vecs;

mod r#impl;

#[cfg(feature = "tokio")]
pub use r#async::*;
pub use r#impl::ResolvedQuery;
pub use vecs::Vecs;

#[derive(Clone)]
pub struct Query(Arc<QueryInner<'static>>);
struct QueryInner<'a> {
    vecs: &'a Vecs<'a>,
    indexer: &'a Indexer<Ro>,
    computer: &'a Computer<Ro>,
    mempool: Option<Mempool>,
    live_oracle: RwLock<Option<(Height, Arc<Oracle>)>>,
}

impl Query {
    pub fn build(indexer: &Indexer, computer: &Computer, mempool: Option<Mempool>) -> Self {
        let indexer = Box::leak(Box::new(indexer.read_only_clone()));
        let computer = Box::leak(Box::new(computer.read_only_clone()));
        let vecs = Box::leak(Box::new(Vecs::build(indexer, computer)));

        Self(Arc::new(QueryInner {
            vecs,
            indexer,
            computer,
            mempool,
            live_oracle: RwLock::new(None),
        }))
    }

    /// Pipeline-safe ceiling: the highest height for which both the
    /// indexer and computer have committed durable data. Backed by
    /// `Indexer::safe_lengths()`, advanced after each complete compute
    /// pass and lowered before any rollback.
    ///
    /// Returns a height (the last fully-written block), not a length.
    /// `safe_lengths().height` is a count: `N` means heights `0..N` are
    /// committed, so the highest is `N-1`. Pre-genesis (`N == 0`) falls
    /// back to `Height::default()` and clients treat it as "nothing
    /// indexed yet".
    pub fn height(&self) -> Height {
        self.safe_lengths().height.decremented().unwrap_or_default()
    }

    /// Snapshot of the pipeline-safe `Lengths`. Hot paths that need
    /// multiple bound fields should call this once at entry and reuse.
    pub(crate) fn safe_lengths(&self) -> Lengths {
        self.indexer().safe_lengths()
    }

    pub(crate) fn read_bounds(&self, safe: Lengths) -> ReadBounds {
        let mut bounds = ReadBounds::new();

        bounds.set(Index::Height.name(), safe.height.into());
        bounds.set(Index::TxIndex.name(), safe.tx_index.into());
        bounds.set(Index::TxInIndex.name(), safe.txin_index.into());
        bounds.set(Index::TxOutIndex.name(), safe.txout_index.into());
        bounds.set(
            Index::EmptyOutputIndex.name(),
            safe.empty_output_index.into(),
        );
        bounds.set(Index::OpReturnIndex.name(), safe.op_return_index.into());
        bounds.set(Index::P2AAddrIndex.name(), safe.p2a_addr_index.into());
        bounds.set(Index::P2MSOutputIndex.name(), safe.p2ms_output_index.into());
        bounds.set(Index::P2PK33AddrIndex.name(), safe.p2pk33_addr_index.into());
        bounds.set(Index::P2PK65AddrIndex.name(), safe.p2pk65_addr_index.into());
        bounds.set(Index::P2PKHAddrIndex.name(), safe.p2pkh_addr_index.into());
        bounds.set(Index::P2SHAddrIndex.name(), safe.p2sh_addr_index.into());
        bounds.set(Index::P2TRAddrIndex.name(), safe.p2tr_addr_index.into());
        bounds.set(Index::P2WPKHAddrIndex.name(), safe.p2wpkh_addr_index.into());
        bounds.set(Index::P2WSHAddrIndex.name(), safe.p2wsh_addr_index.into());
        bounds.set(
            Index::UnknownOutputIndex.name(),
            safe.unknown_output_index.into(),
        );

        let tip = safe.height.decremented();
        bounds.set(
            Index::Epoch.name(),
            tip.map(|height| usize::from(Epoch::from(height)) + 1)
                .unwrap_or(0),
        );
        bounds.set(
            Index::Halving.name(),
            tip.map(|height| usize::from(Halving::from(height)) + 1)
                .unwrap_or(0),
        );

        let timestamp =
            tip.and_then(|height| self.indexer().vecs().blocks.timestamp.collect_one(height));
        for index in Index::all().into_iter().filter(Index::is_date_based) {
            let len = timestamp
                .and_then(|timestamp| index.timestamp_to_index(timestamp))
                .map(|last| last + 1)
                .unwrap_or(0);
            bounds.set(index.name(), len);
        }

        bounds
    }

    /// Tip block hash at the pipeline-safe ceiling.
    #[inline]
    pub fn tip_blockhash(&self) -> BlockHash {
        self.indexer().tip_blockhash()
    }

    /// Tip block hash prefix for cache etags.
    #[inline]
    pub fn tip_hash_prefix(&self) -> BlockHashPrefix {
        BlockHashPrefix::from(&self.tip_blockhash())
    }

    /// Build sync status with the given tip height. `indexed_height` and
    /// `computed_height` reflect live per-vec stamps (diagnostic) and may be
    /// briefly ahead of fully-flushed data; the timestamp data read uses the
    /// safe-lengths-derived height so it never outruns committed bytes.
    pub fn sync_status(&self, tip_height: Height) -> Result<SyncStatus> {
        let indexed_height = self.indexer().indexed_height();
        let computed_height = self.computer().computed_height();
        let blocks_behind = Height::from(tip_height.saturating_sub(*indexed_height));
        let last_indexed_at_unix = self
            .indexer()
            .vecs()
            .blocks
            .timestamp
            .collect_one(self.height())
            .data()?;

        Ok(SyncStatus {
            indexed_height,
            computed_height,
            tip_height,
            blocks_behind,
            last_indexed_at: last_indexed_at_unix.to_iso8601(),
            last_indexed_at_unix,
        })
    }

    #[inline]
    pub fn reader(&self) -> &Reader {
        self.0.indexer.reader()
    }

    #[inline]
    pub fn client(&self) -> &Client {
        self.reader().client()
    }

    #[inline]
    pub fn blocks_dir(&self) -> &Path {
        self.reader().blocks_dir()
    }

    #[inline]
    pub fn indexer(&self) -> &Indexer<Ro> {
        self.0.indexer
    }

    #[inline]
    pub fn computer(&self) -> &Computer<Ro> {
        self.0.computer
    }

    #[inline]
    pub fn mempool(&self) -> Option<&Mempool> {
        self.0.mempool.as_ref()
    }

    #[inline]
    pub fn vecs(&self) -> &'static Vecs<'static> {
        self.0.vecs
    }
}

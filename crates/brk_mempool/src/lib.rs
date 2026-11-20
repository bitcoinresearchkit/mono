//! One private mempool writer and a shared immutable read publication.
//!
//! Fetch, prepare, apply, resolve inputs, and build the graph on the writer.
//! Readers keep the last complete state until one atomic replacement publishes
//! its successor. The stores are copy-on-write (sharded maps, chunked vectors and queues): a
//! publication shares them, and a write copies only the shard or chunk it lands in, so one costs
//! what changed since the last. Transaction bodies are shared and use copy-on-write for fills.

use std::{path::PathBuf, sync::Arc, time::Instant};

use arc_swap::ArcSwap;
use brk_rpc::Client;
use brk_types::{MempoolEntryInfo, Txid};
use rapidhash::RapidHashMap;
use rustc_hash::FxHashMap;

mod api;
pub mod audit;
mod chain_waker;
mod driver;
mod json;
mod read_only;
mod snapshot;
mod state;
mod steps;
mod stores;
mod template;
#[cfg(test)]
mod tests;

pub use api::{
    MempoolBlockSource, PendingUtxos, PendingView, PublishedEntry, PublishedTx, RbfForTx,
    RbfNode, ResolvedMempoolBlockDiff, Spends,
};
pub use chain_waker::ChainWaker;
pub use json::JsonBody;
pub use read_only::ReadOnlyMempool;
use snapshot::Snapshot;
pub use state::ReadOnlyState;
pub use steps::{TxRemoval, prevouts::rpc_resolver};
pub use template::{DatedTemplate, TipFacts};

use snapshot::Rebuilder;
use state::State;
use stores::tx_graveyard::GraveyardFile;

/// Saves the graveyard of the last publication, from any thread (a shutdown's).
#[derive(Clone)]
pub struct GraveyardSaver {
    file: Arc<GraveyardFile>,
    read_only: ReadOnlyMempool,
}

impl GraveyardSaver {
    /// Nothing before the first publication: the last save stands.
    pub fn save(&self) {
        let published = self.read_only.load();
        let Some(pool) = published.pool.as_deref() else {
            return;
        };
        match self.file.save(
            &pool.graveyard,
            |txid| pool.pending_head(txid),
            Some(&pool.txs),
            pool.published_at,
        ) {
            Ok(()) => tracing::info!(
                "Saved mempool graveyard and pool to {}",
                self.file.path().display()
            ),
            Err(error) => tracing::warn!(%error, "mempool graveyard: save failed"),
        }
    }
}

/// Single owner of the mutable pool and update pipeline.
pub struct Mempool {
    client: Client,
    state: State,
    rebuilder: Rebuilder,
    read_only: ReadOnlyMempool,
    unread_cycles: u8,
    /// Completed pairs held until a coherent listing can apply them.
    fetched: RapidHashMap<Txid, (MempoolEntryInfo, bitcoin::Transaction)>,
    /// The last deciding cycle's listed transactions without a matching entry and body, each
    /// with whether it was logged as missed twice in a row.
    missed: FxHashMap<Txid, bool>,
    /// Whether that cycle held its publication back for them.
    held_back: bool,
    chain: ChainWaker,
    /// The last complete observation, withheld until the chain publishes its tip.
    withheld: Option<brk_types::BlockHash>,
    /// Stamp taken at the last successful tip bracket, including withheld observations.
    observed_at: Option<Instant>,
    /// Debug builds' check of each published template against Core, once driven.
    #[cfg(debug_assertions)]
    checker: Option<template::Checker>,
    /// Where the graveyard is kept, and when a save was last tried.
    graveyard_file: Option<(Arc<GraveyardFile>, Instant)>,
}

impl Mempool {
    pub fn new(client: &Client) -> Self {
        Self {
            client: client.clone(),
            state: State::default(),
            rebuilder: Rebuilder::default(),
            read_only: ReadOnlyMempool {
                current: Arc::new(ArcSwap::from_pointee(ReadOnlyState::default())),
                observations: Arc::default(),
                audits: Arc::default(),
            },
            unread_cycles: 0,
            fetched: RapidHashMap::default(),
            missed: FxHashMap::default(),
            held_back: false,
            chain: ChainWaker::default(),
            withheld: None,
            observed_at: None,
            #[cfg(debug_assertions)]
            checker: None,
            graveyard_file: None,
        }
    }

    /// Open permanent observations before starting the driver and HTTP server. The daemon keeps
    /// the sole writer; every read handle shares its immutable records.
    pub fn keep_audits(&mut self, path: &std::path::Path, exit: &brk_exit::Exit) -> brk_error::Result<audit::AuditStore> {
        let audits = audit::AuditStore::open(path, exit)?;
        self.read_only.audits.store(Some(Arc::new(audits.reader())));
        Ok(audits)
    }

    /// Keeps the graveyard (RBF trees, removal reasons, first sightings) at `path` across
    /// restarts: read back now, the last publication's saved every few minutes while it
    /// changes. Save it at shutdown with the returned handle, which saves the pool with it: a
    /// restart then reuses the saved bodies for txids Core still lists.
    pub fn keep_graveyard(&mut self, path: PathBuf) -> GraveyardSaver {
        let file = GraveyardFile::new(path);
        let (graveyard, pool) = file.load();
        self.state = State::restored(graveyard, pool);
        self.graveyard_file = Some((file.clone(), Instant::now()));
        GraveyardSaver {
            file,
            read_only: self.read_only.clone(),
        }
    }

    /// The handle the chain calls after each publication, to wake the driver.
    pub fn chain_waker(&self) -> ChainWaker {
        self.chain.clone()
    }

    pub fn read_only_clone(&self) -> ReadOnlyMempool {
        self.read_only.clone()
    }
}

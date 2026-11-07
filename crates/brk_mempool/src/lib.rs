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
use rapidhash::RapidHashMap;

mod api;
mod chain_waker;
mod driver;
mod json;
mod key;
mod read_only;
mod snapshot;
mod state;
mod steps;
mod stores;
#[cfg(test)]
mod tests;

pub use api::{
    MempoolBlockSource, PendingUtxos, PendingView, PublishedTx, RbfForTx,
    RbfNode, ResolvedMempoolBlockDiff,
};
pub use chain_waker::ChainWaker;
pub use json::JsonBody;
pub use read_only::ReadOnlyMempool;
use snapshot::Snapshot;
pub use state::ReadOnlyState;
pub use steps::{TxRemoval, prevouts::rpc_resolver};

use snapshot::Rebuilder;
use state::State;
use stores::{TxGraveyard, tx_graveyard::GraveyardFile};

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
        match self
            .file
            .save(&pool.graveyard, |txid| pool.pending_head(txid), pool.published_at)
        {
            Ok(()) => tracing::info!("Saved mempool graveyard to {}", self.file.path().display()),
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
    needs_recovery: bool,
    chain: ChainWaker,
    /// The last cycle's observation, withheld at a tip the chain hadn't published yet (and whether
    /// its membership was ready to publish): it goes out the moment the chain publishes that tip.
    withheld: Option<(brk_types::BlockHash, bool)>,
    /// Cycles in a row whose new blocks couldn't all be read.
    unread_cycles: u32,
    /// Listed transactions asked for that didn't load, with how many cycles in a row.
    failing: RapidHashMap<brk_types::Txid, u32>,
    /// Where the graveyard is kept, and when a save was last tried.
    graveyard_file: Option<(Arc<GraveyardFile>, Instant)>,
    /// The graveyard read back at start: what a recovery before the first publication restores.
    loaded_graveyard: TxGraveyard,
}

impl Mempool {
    pub fn new(client: &Client) -> Self {
        Self {
            client: client.clone(),
            state: State::default(),
            rebuilder: Rebuilder::default(),
            read_only: ReadOnlyMempool {
                current: Arc::new(ArcSwap::from_pointee(ReadOnlyState::default())),
            },
            needs_recovery: false,
            chain: ChainWaker::default(),
            withheld: None,
            unread_cycles: 0,
            failing: RapidHashMap::default(),
            graveyard_file: None,
            loaded_graveyard: TxGraveyard::default(),
        }
    }

    /// Keeps the graveyard (RBF trees, removal reasons, first sightings) at `path` across
    /// restarts: read back now, the last publication's saved every few minutes while it
    /// changes. Save it at shutdown with the returned handle.
    pub fn keep_graveyard(&mut self, path: PathBuf) -> GraveyardSaver {
        let file = GraveyardFile::new(path);
        self.loaded_graveyard = file.load();
        self.state.graveyard = self.loaded_graveyard.clone();
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

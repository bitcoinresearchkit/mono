#![allow(clippy::type_complexity)]

//! One private mempool writer and a shared immutable read publication.
//!
//! Fetch, prepare, apply, resolve inputs, and build the graph on the writer.
//! Readers keep the last complete state until one atomic replacement publishes
//! its successor. Transaction bodies are shared and use copy-on-write for fills.
//!

use std::sync::Arc;

use arc_swap::ArcSwap;
use brk_rpc::Client;

mod api;
mod chain_waker;
mod driver;
mod read_only;
mod snapshot;
mod state;
mod steps;
mod stores;

pub use api::{
    BlockTemplateDiffJson, BlockTemplateJson, BlockTemplateSource, PendingUtxos, PublishedTx,
    RbfForTx, RbfNode, ResolvedBlockTemplateDiff,
};
pub use chain_waker::ChainWaker;
pub use read_only::ReadOnlyMempool;
use snapshot::Snapshot;
pub use state::ReadOnlyState;
use steps::TxRemoval;

use snapshot::Rebuilder;
use state::State;

/// Single owner of the mutable pool and update pipeline.
///
pub struct Mempool {
    client: Client,
    state: State,
    rebuilder: Rebuilder,
    read_only: ReadOnlyMempool,
    needs_recovery: bool,
    chain: ChainWaker,
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

#![allow(clippy::type_complexity)]

//! One private mempool writer and a shared immutable read publication.
//!
//! Fetch, prepare, apply, resolve inputs, and build the graph on the writer.
//! Readers keep the last complete state until one atomic replacement publishes
//! its successor. Transaction bodies are shared and use copy-on-write for fills.
//!
//! ```no_run
//! use brk_mempool::Mempool;
//! # fn make_client() -> brk_rpc::Client { unimplemented!() }
//! let client = make_client();
//! let mut mempool = Mempool::new(&client);
//! let reader = mempool.read_only_clone();
//! std::thread::spawn(move || mempool.start());
//! let state = reader.load();
//! let info = state.info();
//! ```

use std::sync::Arc;

use arc_swap::ArcSwap;
use brk_rpc::Client;

mod api;
mod cycle;
mod diagnostics;
mod driver;
mod read_only;
mod snapshot;
mod state;
mod steps;
mod stores;

#[cfg(test)]
mod test_support;

pub use api::{BlockTemplateSource, RbfForTx, RbfNode, ResolvedBlockTemplateDiff};
pub use cycle::{Cycle, TxAdded, TxRemoved};
pub use diagnostics::MempoolStats;
pub use read_only::ReadOnlyMempool;
pub use snapshot::Snapshot;
pub use state::ReadOnlyState;
pub use steps::TxRemoval;

use snapshot::Rebuilder;
use state::State;

/// Single owner of the mutable pool and update pipeline.
///
/// ```compile_fail
/// # use brk_mempool::Mempool;
/// fn duplicate(writer: &Mempool) -> Mempool { writer.clone() }
/// ```
pub struct Mempool {
    client: Client,
    state: State,
    rebuilder: Rebuilder,
    read_only: ReadOnlyMempool,
    needs_recovery: bool,
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
        }
    }

    pub fn read_only_clone(&self) -> ReadOnlyMempool {
        self.read_only.clone()
    }

    /// Working-cycle counters, for the update owner and CLI.
    fn stats(&self) -> MempoolStats {
        MempoolStats::from(self)
    }
}

#[cfg(test)]
#[path = "../tests/unit/lib_test_helpers.rs"]
mod test_helpers;

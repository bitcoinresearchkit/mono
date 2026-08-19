#![allow(clippy::type_complexity)]

//! Live mempool monitor for the brk indexer.
//!
//! One pull cycle, five steps:
//!
//! ```text
//!   Fetcher  ->  Preparer  ->  Applier  ->  Prevouts  ->  Rebuilder
//!     RPC        decode &     write to       fill         build
//!                classify     State          missing      Snapshot
//!                                            prevouts
//! ```
//!
//! 1. [`steps::Fetcher`] - one mixed batched RPC for
//!    `getblocktemplate` + `getrawmempool false` + `getmempoolinfo`,
//!    then a single mixed `getmempoolentry`+`getrawtransaction` batch
//!    on new txids only. GBT-only txs are synthesized inline from the
//!    GBT payload so block 0 matches Core's selection exactly without
//!    a follow-up entry fetch that could race the listing.
//! 2. [`steps::Preparer`] - decode and classify into
//!    `TxsPulled { added, removed }`. Pure CPU.
//! 3. [`steps::Applier`] - apply the diff to [`state::State`] under a
//!    single write lock.
//! 4. [`steps::Prevouts::fill`] - fills `prevout: None` inputs in one
//!    pass, using same-cycle in-mempool parents directly and the
//!    caller-supplied resolver (default: `getrawtransaction`) for
//!    confirmed parents.
//! 5. [`snapshot::Rebuilder`] - rebuilds the projected-blocks
//!    [`Snapshot`] from the same-cycle GBT and min fee.
//!
//! # Locking domains
//!
//! Two independent locks. No path holds both simultaneously.
//!
//! - `State` (`RwLock<State>`): the live mempool. Cycle steps 3 and 4
//!   take the write guard. Every read-side accessor takes a read guard.
//! - `Rebuilder.{snapshot, history}` (two `RwLock`s, written in that
//!   order each cycle): the published projection. Readers grab one or
//!   the other. The cycle drops its `State` guard before touching them.
//!
//! # Usage
//!
//! Drive the loop on a worker thread and read from any clone:
//!
//! ```no_run
//! use brk_mempool::Mempool;
//! # fn make_client() -> brk_rpc::Client { unimplemented!() }
//! let client = make_client();
//! let mempool = Mempool::new(&client);
//! let reader = mempool.clone();
//! std::thread::spawn(move || mempool.start());
//! // `reader.snapshot()`, `reader.block_template()`, etc. on this thread.
//! # let _ = reader;
//! ```
//!
//! A `Mempool` hosts at most one driver. Calling `start` / `start_with`
//! a second time on the same instance panics. Spawn a separate
//! `Mempool::new` if you need more loops.

use std::sync::{Arc, atomic::AtomicBool};

use brk_rpc::Client;
use parking_lot::{RwLock, RwLockReadGuard};

mod api;
mod cycle;
mod diagnostics;
mod driver;
mod snapshot;
mod state;
mod steps;
mod stores;

#[cfg(test)]
mod test_support;

pub use api::{RbfForTx, RbfNode};
pub use cycle::{AddedKind, Cycle, TxAdded, TxRemoved};
pub use diagnostics::MempoolStats;
pub use snapshot::Snapshot;
pub use steps::TxRemoval;

use snapshot::Rebuilder;
use state::State;

/// Cheaply cloneable: clones share one live mempool via `Arc`.
#[derive(Clone)]
pub struct Mempool(Arc<Inner>);

struct Inner {
    client: Client,
    state: RwLock<State>,
    rebuilder: Rebuilder,
    started: AtomicBool,
}

impl Mempool {
    pub fn new(client: &Client) -> Self {
        Self(Arc::new(Inner {
            client: client.clone(),
            state: RwLock::new(State::default()),
            rebuilder: Rebuilder::default(),
            started: AtomicBool::new(false),
        }))
    }

    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.0.rebuilder.snapshot()
    }

    /// One-shot diagnostic counters captured under a single read guard.
    pub fn stats(&self) -> MempoolStats {
        MempoolStats::from(self)
    }

    fn rebuilder(&self) -> &Rebuilder {
        &self.0.rebuilder
    }

    fn read(&self) -> RwLockReadGuard<'_, State> {
        self.0.state.read()
    }
}

#[cfg(test)]
mod test_helpers {
    use brk_rpc::Auth;
    use brk_types::{FeeRate, Txid};

    use super::*;

    impl Mempool {
        /// Test-only constructor that wires a Client at the default URL without
        /// touching the network. `simple_http` only parses the URL on init.
        pub(crate) fn for_test() -> Self {
            let client = Client::new(Client::default_url(), Auth::None).unwrap();
            Self(Arc::new(Inner {
                client,
                state: RwLock::new(State::default()),
                rebuilder: Rebuilder::default(),
                started: AtomicBool::new(false),
            }))
        }

        pub(crate) fn test_state_lock(&self) -> &RwLock<State> {
            &self.0.state
        }

        pub(crate) fn test_tick(&self, gbt_txids: &[Txid], min_fee: FeeRate) {
            self.0.rebuilder.tick(&self.0.state, gbt_txids, min_fee);
        }
    }
}

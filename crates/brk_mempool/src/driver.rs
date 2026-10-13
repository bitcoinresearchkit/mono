//! The sole update owner prepares data privately and publishes once per cycle.

use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use brk_error::{Error, Result};
use brk_types::{BlockHash, TxOut, Txid, Vout};
use rustc_hash::FxHashMap;
use tracing::error;

use crate::{
    Mempool, State,
    cycle::{Cycle, CycleDiff},
    steps::{Fetched, applier, fetcher, preparer, prevouts},
};

const PERIOD: Duration = Duration::from_millis(1000);
/// How often a withheld pool checks whether the chain has published its tip.
const WITHHELD_POLL: Duration = Duration::from_millis(5);

impl Mempool {
    /// Drive one update per second. Overrunning cycles resume immediately.
    /// The exclusive writer borrow prevents concurrent drivers or manual ticks.
    ///
    /// A pool is published only at the tip the chain has published (`published_tip`), so readers
    /// pair it with their chain view; one withheld for a newer tip goes out right after the chain
    /// publishes that tip.
    pub fn start_with<F>(&mut self, resolver: F, published_tip: impl Fn() -> BlockHash)
    where
        F: Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut> + Send,
    {
        let gate = || Some(published_tip());
        loop {
            let started = Instant::now();
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                if let Err(e) = self.tick_with(&resolver, gate) {
                    error!("mempool update failed: {e}");
                }
            }));
            if let Err(payload) = outcome {
                error!(
                    "mempool update panicked; restoring completed state: {}",
                    Self::panic_msg(&payload)
                );
            }
            while let Some(rest) = PERIOD.checked_sub(started.elapsed()) {
                match self.withheld {
                    Some(tip) if tip == published_tip() => break,
                    Some(_) => thread::sleep(rest.min(WITHHELD_POLL)),
                    None => thread::sleep(rest),
                }
            }
        }
    }

    /// One update published at the node's tip, for drivers without a chain to follow.
    pub fn tick(&mut self) -> Result<Cycle> {
        self.tick_with(prevouts::rpc_resolver(self.client.clone()), || None)
    }

    /// Resolve confirmed parents with `resolver`; mempool parents are filled
    /// internally. Failed observations leave the public state intact. Applied
    /// changes still appear in Cycle, even when a final observation fails.
    fn tick_with<F>(&mut self, resolver: F, gate: impl Fn() -> Option<BlockHash>) -> Result<Cycle>
    where
        F: Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut>,
    {
        if self.needs_recovery {
            self.restore_published();
        }
        let cycle = self.tick_once(resolver, gate)?;
        self.needs_recovery = false;
        Ok(cycle)
    }

    fn restore_published(&mut self) {
        let published = self.read_only.load();
        self.state = published
            .pool
            .as_ref()
            .map_or_else(State::default, |pool| pool.restore());
        self.rebuilder.restore(
            published
                .pool
                .as_ref()
                .map_or_else(Arc::default, |pool| pool.graph.clone()),
        );
        self.needs_recovery = false;
    }

    fn tick_once<F>(&mut self, resolver: F, gate: impl Fn() -> Option<BlockHash>) -> Result<Cycle>
    where
        F: Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut>,
    {
        let started = Instant::now();
        // RPC batches are not atomic. Bracket fetch and resolution with full tips.
        let tip_before = self.client.get_best_block_hash()?;
        let Fetched {
            state: rpc,
            new_entries,
            new_txs,
            block_template_txids,
            address_view_complete,
        } = fetcher::fetch(&self.client, &self.state)?;
        if rpc.tip_hash != tip_before {
            return Err(Error::StateUpdating);
        }
        let pulled = preparer::prepare(&rpc.live_txids, new_entries, new_txs, &self.state);
        let mut diff = CycleDiff::default();
        // A panic after this point requires restoring private indexes before reuse.
        self.needs_recovery = true;
        applier::apply(
            &mut self.state,
            &self.rebuilder.snapshot(),
            pulled,
            &mut diff,
        );
        prevouts::fill(&mut self.state, &mut diff, resolver);
        let coherent_tip = self.client.get_best_block_hash().ok() == Some(tip_before);
        self.rebuilder
            .tick(&self.state, &block_template_txids, rpc.min_fee);
        if coherent_tip {
            // Never ahead of the chain readers pair the pool with.
            if gate().is_none_or(|published| published == rpc.tip_hash) {
                self.publish_observation(
                    rpc.tip_hash,
                    address_view_complete && self.state.contains_all(&rpc.live_txids),
                );
                self.withheld = None;
            } else {
                self.withheld = Some(rpc.tip_hash);
            }
        }
        let CycleDiff {
            added,
            removed,
            addrs,
        } = diff;
        let (addr_enters, addr_leaves) = addrs.into_vecs();
        Ok(Cycle {
            added,
            removed,
            addr_enters,
            addr_leaves,
            tip_hash: rpc.tip_hash,
            tip_height: rpc.tip_height,
            info: self.state.info.clone(),
            snapshot: self.rebuilder.snapshot(),
            took: started.elapsed(),
        })
    }

    fn publish_observation(&mut self, tip: BlockHash, membership_complete: bool) {
        let previous = self.read_only.load();
        if let Some(next) = previous.updated(
            &self.state,
            tip,
            self.rebuilder.snapshot(),
            membership_complete,
        ) {
            self.read_only.current.store(Arc::new(next));
        }
    }

    fn panic_msg(payload: &(dyn Any + Send)) -> &str {
        payload
            .downcast_ref::<&'static str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("<non-string panic payload>")
    }
}

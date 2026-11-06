//! The sole update owner prepares data privately and publishes once per cycle.

use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
    time::{Duration, Instant},
};

use brk_error::{Error, Result};
use brk_types::{BlockHash, TxOut, Txid, Vout};
use rapidhash::{RapidHashMap, RapidHashSet};
use tracing::{error, info, warn};

use crate::{
    Mempool, State,
    steps::{
        Confirmed, Fetched, applier,
        fetcher::{self, MAX_TX_FETCHES_PER_CYCLE},
        preparer, prevouts,
    },
};

const PERIOD: Duration = Duration::from_millis(1000);

impl Mempool {
    /// Drive one update per second. Overrunning cycles resume immediately.
    /// The exclusive writer borrow prevents concurrent drivers or manual ticks.
    ///
    /// A pool is published only at the tip the chain has published (`published_tip`), so readers
    /// pair it with their chain view; when the chain publishes a tip and wakes the driver
    /// ([`Self::chain_waker`]), the last cycle's observation at that tip goes out at once, or a
    /// cycle starts at once for a pool behind it.
    pub fn start_with<F>(&mut self, resolver: F, published_tip: impl Fn() -> BlockHash)
    where
        F: Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut> + Send,
    {
        let gate = || Some(published_tip());
        let loading = Instant::now();
        loop {
            let started = Instant::now();
            let loaded = self.read_only.load().tip().is_some();
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                // The chain caught up with a withheld observation: out at once, no cycle needed.
                if let Some((tip, membership_complete)) =
                    self.withheld.take_if(|(tip, _)| gate() == Some(*tip))
                {
                    return self.publish_observation(tip, membership_complete);
                }
                match self.tick_with(&resolver, gate) {
                    // A block arrived mid-fetch: the next cycle observes the new tip.
                    Ok(()) | Err(Error::StateUpdating) => {}
                    Err(e) => error!("mempool update failed: {e}"),
                }
            }));
            if let Err(payload) = outcome {
                error!(
                    "mempool update panicked; restoring completed state: {}",
                    Self::panic_msg(&payload)
                );
            }
            if !loaded && let Some(pool) = &self.read_only.load().pool {
                info!(
                    "Loaded mempool ({} transactions) in {:?}",
                    pool.info.count,
                    loading.elapsed()
                );
            }
            while let Some(rest) = PERIOD.checked_sub(started.elapsed()) {
                if self.chain.wait(rest) && self.read_only.load().tip() != Some(published_tip()) {
                    break;
                }
            }
        }
    }

    /// One update published at the node's tip, for drivers without a chain to follow.
    /// `StateUpdating` when it published nothing: the tip moved mid-cycle (a block arrived, or
    /// couldn't be read back), or the node is reloading its mempool.
    pub fn tick(&mut self) -> Result<()> {
        self.tick_with(prevouts::rpc_resolver(self.client.clone()), || None)
    }

    /// Resolve confirmed parents with `resolver`; mempool parents are filled
    /// internally. Failed observations leave the public state intact.
    fn tick_with<F>(&mut self, resolver: F, gate: impl Fn() -> Option<BlockHash>) -> Result<()>
    where
        F: Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut>,
    {
        if self.needs_recovery {
            self.restore_published();
        }
        let coherent = self.tick_once(resolver, gate)?;
        // Applied either way: a cycle whose tip moved at its end publishes nothing, the next one
        // builds on it.
        self.needs_recovery = false;
        if coherent {
            Ok(())
        } else {
            Err(Error::StateUpdating)
        }
    }

    fn restore_published(&mut self) {
        self.withheld = None;
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
        // The blocks after the restored pool's tip are still to classify its removals by.
        self.last_tip = published.pool.as_ref().map(|pool| pool.tip);
        self.needs_recovery = false;
    }

    /// Whether the cycle's tip held to its end (only then can it publish).
    fn tick_once<F>(&mut self, resolver: F, gate: impl Fn() -> Option<BlockHash>) -> Result<bool>
    where
        F: Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut>,
    {
        // RPC batches are not atomic. Bracket fetch and resolution with full tips.
        let tip_before = self.client.get_best_block_hash()?;
        let Fetched {
            state: rpc,
            new_entries,
            new_txs,
            block_template_txids,
            template_only,
            capped,
        } = fetcher::fetch(&self.client, &self.state)?;
        // The first load fills the cap every cycle until the pool publishes; after it, a full
        // cycle means a flood of new transactions.
        if capped && self.read_only.load().tip().is_some() {
            warn!(
                cap = MAX_TX_FETCHES_PER_CYCLE,
                "mempool new-tx fetch hit the per-cycle cap"
            );
        }
        // A block arriving during the fetch can leave the listing past the template's tip: nothing
        // of such a cycle is applied (its removals would be classified without that block). Nor
        // while the node reloads its saved mempool: its listing lacks what it hasn't read back yet,
        // which would read as gone (readers keep the last publication meanwhile).
        if !rpc.loaded
            || rpc.tip_hash != tip_before
            || self.client.get_best_block_hash()? != tip_before
        {
            return Err(Error::StateUpdating);
        }
        let confirmed = Confirmed::since(&self.client, rpc.tip_hash, self.last_tip);
        let diff = preparer::prepare(
            &rpc.live_txids,
            &template_only,
            new_entries,
            new_txs,
            &confirmed,
            &self.state,
        );
        // A panic after this point requires restoring private indexes before reuse. The private
        // state changes from here: what was withheld no longer is it.
        self.withheld = None;
        self.needs_recovery = true;
        applier::apply(&mut self.state, self.rebuilder.snapshot(), diff, &confirmed);
        if !confirmed.incomplete {
            self.last_tip = Some(rpc.tip_hash);
        }
        self.state.template_only = template_only;
        prevouts::fill(&mut self.state, resolver);
        let coherent_tip = self.client.get_best_block_hash().ok() == Some(tip_before);
        // The template and the listing agree, and every listed transaction is loaded.
        let membership_complete =
            self.state.template_only.is_empty() && self.state.contains_all(&rpc.live_txids);
        let unloaded: RapidHashSet<Txid> = if membership_complete {
            RapidHashSet::default()
        } else {
            rpc.live_txids
                .iter()
                .filter(|txid| !self.state.txs.contains(txid))
                .copied()
                .collect()
        };
        self.rebuilder.tick(
            &self.state,
            block_template_txids.as_deref(),
            rpc.min_fee,
            &rpc.fee_deltas,
            &unloaded,
        );
        if coherent_tip {
            // Never ahead of the chain readers pair the pool with.
            if gate().is_none_or(|published| published == rpc.tip_hash) {
                self.publish_observation(rpc.tip_hash, membership_complete);
            } else {
                self.withheld = Some((rpc.tip_hash, membership_complete));
            }
        }
        Ok(coherent_tip)
    }

    fn publish_observation(&mut self, tip: BlockHash, membership_complete: bool) {
        let previous = self.read_only.load();
        if let Some(next) = previous.updated(
            &self.state,
            tip,
            self.rebuilder.snapshot().clone(),
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

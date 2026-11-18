//! The sole update owner prepares data privately and publishes once per cycle.

use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use brk_error::{Error, Result};
use brk_rpc::MempoolObservation;
use brk_types::{BlockHash, TxOut, Txid, Vout};
use rapidhash::RapidHashMap;
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::{error, info, warn};

use crate::{
    Mempool, State, TipFacts,
    key::TxKey,
    steps::{Confirmed, applier, preparer, prevouts},
};

const PERIOD: Duration = Duration::from_millis(1000);
/// Bound the cycles waiting for a block Core cannot return.
const MAX_UNREAD_CYCLES: u8 = 10;
/// How often a changed graveyard is saved (`Mempool::keep_graveyard`): a crash loses at most this
/// much of it.
const SAVE_EVERY: Duration = Duration::from_secs(300);

impl Mempool {
    /// Drive one update per second. Overrunning cycles resume immediately.
    /// The exclusive writer borrow prevents concurrent drivers or manual ticks.
    ///
    /// A pool is published only at the tip the chain has published (`published_tip`), so readers
    /// pair it with their chain view; when the chain publishes a tip and wakes the driver
    /// ([`Self::chain_waker`]), the last cycle's observation at that tip goes out at once, or a
    /// cycle starts at once for a pool behind it. `tip_facts`: what the chain says of a tip it
    /// published, for the next block's template, built with each projection.
    pub fn start_with<F>(
        &mut self,
        resolver: F,
        published_tip: impl Fn() -> BlockHash,
        tip_facts: impl Fn(&BlockHash) -> Option<TipFacts>,
    ) where
        F: Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut> + Send,
    {
        #[cfg(debug_assertions)]
        {
            self.checker = Some(crate::template::Checker::spawn(self.client.clone()));
        }
        let gate = || Some(published_tip());
        let loading = Instant::now();
        loop {
            let started = Instant::now();
            let loaded = self.read_only.load().tip().is_some();
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                // The chain caught up with a withheld observation: out at once, no cycle needed.
                if let Some(tip) = self.withheld.take_if(|tip| gate() == Some(*tip)) {
                    return self.publish_observation(tip, &tip_facts);
                }
                match self.tick_with(&resolver, gate, &tip_facts) {
                    // A block arrived mid-fetch, or a newly missed transaction held the cycle
                    // back: the next cycle observes again.
                    Ok(()) | Err(Error::StateUpdating(_)) => {}
                    Err(e) => error!("mempool update failed: {e}"),
                }
            }));
            if let Err(payload) = outcome {
                error!(
                    "mempool update panicked; restoring completed state: {}",
                    Self::panic_msg(&payload)
                );
            }
            self.save_graveyard_if_due();
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
    /// couldn't be read back), a newly missed transaction held the cycle back, or the node is
    /// reloading its mempool.
    pub fn tick(&mut self) -> Result<()> {
        self.tick_with(
            prevouts::rpc_resolver(self.client.clone()),
            || None,
            |_| None,
        )
    }

    /// Resolve confirmed parents with `resolver`; mempool parents are filled
    /// internally. Failed observations leave the public state intact.
    fn tick_with<F>(
        &mut self,
        resolver: F,
        gate: impl Fn() -> Option<BlockHash>,
        tip_facts: impl Fn(&BlockHash) -> Option<TipFacts>,
    ) -> Result<()>
    where
        F: Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut>,
    {
        if self.needs_recovery {
            self.restore_published();
        }
        let coherent = self.tick_once(resolver, gate, tip_facts)?;
        // Applied either way: a cycle whose tip moved at its end publishes nothing, the next one
        // builds on it.
        self.needs_recovery = false;
        if coherent {
            Ok(())
        } else {
            Err(Error::updating())
        }
    }

    fn restore_published(&mut self) {
        self.withheld = None;
        let published = self.read_only.load();
        // Before the first publication, the graveyard read back, if any: the pool reloads from
        // Core, as after a crash.
        self.state = published.pool.as_ref().map_or_else(
            || State::restored(self.loaded_graveyard.clone(), Vec::new()),
            |pool| pool.restore(),
        );
        self.rebuilder.restore(
            published
                .pool
                .as_ref()
                .map_or_else(Arc::default, |pool| pool.graph.clone()),
        );
        // The restored graveyard carries its confirmed tip, so the next cycle reads the blocks
        // after it and classifies removals by them.
        self.needs_recovery = false;
    }

    /// Whether the cycle's tip held to its end (only then can it publish).
    fn tick_once<F>(
        &mut self,
        resolver: F,
        gate: impl Fn() -> Option<BlockHash>,
        tip_facts: impl Fn(&BlockHash) -> Option<TipFacts>,
    ) -> Result<bool>
    where
        F: Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut>,
    {
        let MempoolObservation {
            state: rpc,
            modified_fees,
        } = self
            .client
            .fetch_mempool_state(&self.rebuilder.snapshot().modified_fees)?;
        if !rpc.loaded {
            self.read_only.observations.clear();
            self.observed_at = None;
            return Err(Error::updating());
        }
        let tip = rpc.tip_hash;
        let mut live: FxHashSet<_> = rpc.live_txids.iter().map(TxKey::from).collect();
        self.fetched
            .retain(|txid, _| live.contains(&TxKey::from(txid)) && !self.state.txs.contains(txid));
        let confirmed = self.confirmed(tip)?;
        let requested: Vec<_> = rpc
            .live_txids
            .into_iter()
            .filter(|txid| !self.state.txs.contains(txid) && !self.fetched.contains_key(txid))
            .collect();
        self.client.fetch_new_pool_data(&requested, |entry, body| {
            self.fetched.insert(entry.txid, (entry, body));
        })?;
        // Core reads the tip before its listing. Check it again before applying that listing.
        if self.client.get_best_block_hash()? != tip {
            return Err(Error::updating());
        }
        let missed: Vec<_> = requested
            .into_iter()
            .filter(|txid| !self.fetched.contains_key(txid))
            .collect();
        self.hold_back_for(&missed)?;

        let omitted = self.omit_missed(&missed, &mut live);
        let new = self
            .fetched
            .extract_if(|txid, _| omitted.is_empty() || !omitted.contains(&TxKey::from(txid)))
            .map(|(_, pair)| pair);
        let diff = preparer::prepare(&live, new, &confirmed, &self.state);
        // A panic after this point requires restoring private indexes before reuse.
        self.withheld = None;
        self.needs_recovery = true;
        if !diff.removed.is_empty() && self.rebuilder.snapshot().ensure_projection().is_err() {
            // A shutdown save restores transaction records; rebuild their chunks before burying them.
            self.rebuilder
                .tick(&self.state, rpc.min_fee, &modified_fees);
        }
        let listed = |txid: &Txid| {
            let key = TxKey::from(txid);
            live.contains(&key) || omitted.contains(&key)
        };
        let state = &mut self.state;
        let snapshot = self.rebuilder.snapshot();
        applier::apply(state, snapshot, diff, &confirmed, listed);
        state.graveyard.set_confirmed_tip(tip, &mut state.upkeep);
        state.ancestry.follow(tip, confirmed.path());
        self.unread_cycles = 0;
        prevouts::fill(state, resolver);
        let coherent = self.client.get_best_block_hash().ok() == Some(tip);
        let observed_at = Instant::now();
        self.rebuilder
            .tick(&self.state, rpc.min_fee, &modified_fees);
        if coherent {
            self.observed_at = Some(observed_at);
            if gate().is_none_or(|published| published == tip) {
                self.publish_observation(tip, tip_facts);
            } else {
                self.withheld = Some(tip);
            }
        }
        Ok(coherent)
    }

    fn confirmed(&mut self, tip: BlockHash) -> Result<Confirmed> {
        let confirmed = Confirmed::since(&self.client, tip, self.state.graveyard.confirmed_tip());
        if let Some(error) = &confirmed.error {
            self.unread_cycles = self.unread_cycles.saturating_add(1);
            if self.unread_cycles == 1 {
                warn!(%error, "mempool: retrying unreadable blocks");
            }
            if self.unread_cycles < MAX_UNREAD_CYCLES {
                return Err(Error::updating());
            }
            if self.unread_cycles == MAX_UNREAD_CYCLES {
                warn!(%error, "mempool: continuing after ten cycles with unreadable blocks");
            }
        } else {
            self.unread_cycles = 0;
        }
        Ok(confirmed)
    }

    /// Records this cycle's `missed` (listed, without a matching entry and body) and holds its
    /// publication back (`StateUpdating`) for a newly missed one, so a replacement chain caught
    /// mid-fetch (A→B→C) settles on the next listing; never two cycles in a row, so neither a
    /// transaction that never pairs nor churn holds publication back longer. Item failures are
    /// logged at debug as they come (Core's -5, gone since the listing, is no failure).
    /// One missed in two deciding cycles in a row is logged once, however long it stays missed.
    fn hold_back_for(&mut self, missed: &[Txid]) -> Result<()> {
        let last = std::mem::take(&mut self.missed);
        let mut new = false;
        let mut again = Vec::new();
        for txid in missed {
            let logged = match last.get(txid) {
                None => {
                    new = true;
                    false
                }
                Some(&logged) => {
                    if !logged {
                        again.push(*txid);
                    }
                    true
                }
            };
            self.missed.insert(*txid, logged);
        }
        if let Some(first) = again.first() {
            warn!(
                txid = %first,
                count = again.len(),
                "mempool: listed transactions missed twice in a row are left out, with their descendants, until they pair"
            );
        }
        self.held_back = new && !self.held_back;
        if self.held_back {
            Err(Error::updating())
        } else {
            Ok(())
        }
    }

    /// Takes this cycle's misses and their descendants out of `live`, returning them. Only held
    /// pairs are walked: a pooled transaction is already published and stays, whatever its
    /// parent; a held child of a miss waits with its pair, to enter once the parent pairs.
    fn omit_missed(&self, missed: &[Txid], live: &mut FxHashSet<TxKey>) -> FxHashSet<TxKey> {
        let mut omitted = FxHashSet::default();
        if missed.is_empty() {
            return omitted;
        }
        let mut children: FxHashMap<TxKey, Vec<TxKey>> = FxHashMap::default();
        for (txid, (_, tx)) in &self.fetched {
            for input in &tx.input {
                children
                    .entry(TxKey::from(&Txid::from(input.previous_output.txid)))
                    .or_default()
                    .push(TxKey::from(txid));
            }
        }
        let mut stack: Vec<_> = missed.iter().map(TxKey::from).collect();
        while let Some(key) = stack.pop() {
            if live.remove(&key) {
                omitted.insert(key);
                stack.extend(children.remove(&key).into_iter().flatten());
            }
        }
        omitted
    }

    /// Saves the last publication's graveyard when it changed since the last save that
    /// succeeded, every [`SAVE_EVERY`], on a thread of its own (the publication shares it). A
    /// publication, as the shutdown save's: the later one always wins.
    fn save_graveyard_if_due(&mut self) {
        let Some((file, tried)) = &mut self.graveyard_file else {
            return;
        };
        let published = self.read_only.load();
        let Some(pool) = published.pool.clone() else {
            return;
        };
        if tried.elapsed() < SAVE_EVERY || file.saved_revision() == Some(pool.graveyard.revision())
        {
            return;
        }
        *tried = Instant::now();
        let file = file.clone();
        thread::spawn(move || {
            if let Err(error) = file.save(
                &pool.graveyard,
                |txid| pool.pending_head(txid),
                None,
                pool.published_at,
            ) {
                warn!(%error, "mempool graveyard: save failed");
            }
        });
    }

    /// Publishes membership and projection together, with block 0's template built from
    /// `tip_facts`; in debug builds Core checks it on a thread of its own, publication never
    /// waiting on it.
    fn publish_observation(
        &mut self,
        tip: BlockHash,
        tip_facts: impl FnOnce(&BlockHash) -> Option<TipFacts>,
    ) {
        let previous = self.read_only.load();
        let snapshot = self.rebuilder.snapshot().clone();
        if let Some(next) = previous.updated(&self.state, tip, snapshot, tip_facts) {
            self.read_only.current.store(Arc::new(next));
        }
        if let Some(at) = self.observed_at {
            self.read_only.observations.publish(at);
        }
        #[cfg(debug_assertions)]
        if let (Some(checker), Some(template)) = (&self.checker, &self.read_only.load().template) {
            checker.check(template.clone());
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

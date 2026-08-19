//! Cycle loop. `start_with` drives [`Mempool::tick_with`] every
//! [`PERIOD`]. Each cycle is wrapped in `catch_unwind` so a panic
//! doesn't freeze the snapshot. `parking_lot` locks don't poison.

use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use brk_types::{TxOut, Txid, Vout};
use rustc_hash::FxHashMap;
use tracing::error;

use crate::{
    Inner, Mempool,
    cycle::{Cycle, CycleDiff},
    steps::{Applier, Fetched, Fetcher, Preparer, Prevouts},
};

const PERIOD: Duration = Duration::from_millis(1000);

impl Mempool {
    /// Infinite update loop with a 1s interval. Resolves
    /// confirmed-parent prevouts via the default `getrawtransaction`
    /// resolver. Requires bitcoind started with `txindex=1`. Discards
    /// per-cycle [`Cycle`] events - use [`Mempool::tick`] to consume them.
    pub fn start(&self) {
        self.start_with(Prevouts::rpc_resolver(self.0.client.clone()));
    }

    /// Variant of `start` that uses a caller-supplied resolver for
    /// confirmed-parent prevouts (typically backed by an indexer).
    ///
    /// Sleep is `PERIOD - work_duration`, so a 350ms cycle followed by
    /// a 100ms cycle still ticks roughly every `PERIOD`. When work
    /// overruns `PERIOD`, the next cycle starts immediately.
    ///
    /// # Panics
    ///
    /// Panics if a driver is already running on this `Mempool` instance.
    /// One `Mempool` may host at most one driver. Spawn another instance
    /// for additional loops.
    pub fn start_with<F>(&self, resolver: F)
    where
        F: Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut> + Send,
    {
        if self
            .0
            .started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            panic!("Mempool::start_with already running on this instance");
        }
        loop {
            let started = Instant::now();
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                if let Err(e) = self.tick_with(&resolver) {
                    error!("update failed: {e}");
                }
            }));
            if let Err(payload) = outcome {
                error!(
                    "mempool update panicked, continuing loop: {}",
                    Self::panic_msg(&payload)
                );
            }
            if let Some(rest) = PERIOD.checked_sub(started.elapsed()) {
                thread::sleep(rest);
            }
        }
    }

    /// One sync cycle: fetch, prepare, apply, fill prevouts, rebuild.
    /// Returns a [`Cycle`] reporting everything that changed. Uses the
    /// default `getrawtransaction` resolver for confirmed-parent
    /// prevouts (requires `txindex=1`).
    ///
    /// # Errors
    ///
    /// Propagates any failure from the initial RPC fetch (network drop,
    /// auth, bitcoind error). Steps after `Fetcher::fetch` are infallible
    /// today. The resolver itself swallows its own errors and retries
    /// next cycle.
    pub fn tick(&self) -> brk_error::Result<Cycle> {
        self.tick_with(Prevouts::rpc_resolver(self.0.client.clone()))
    }

    /// Variant of [`Mempool::tick`] with a caller-supplied resolver for
    /// confirmed-parent prevouts. The resolver MUST resolve confirmed
    /// prevouts only. Mempool-to-mempool chains are wired internally
    /// and the resolver is never called for them.
    ///
    /// # Errors
    ///
    /// Same as [`Mempool::tick`]: only the RPC fetch is fallible.
    pub fn tick_with<F>(&self, resolver: F) -> brk_error::Result<Cycle>
    where
        F: Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut>,
    {
        let started = Instant::now();
        let Inner {
            client,
            state,
            rebuilder,
            ..
        } = &*self.0;

        let Fetched {
            state: rpc,
            new_entries,
            new_txs,
            block_template_txids,
        } = Fetcher::fetch(client, state)?;
        let pulled = Preparer::prepare(&rpc.live_txids, new_entries, new_txs, state);
        let mut diff = CycleDiff::default();
        let prev_snapshot = rebuilder.snapshot();
        Applier::apply(state, &prev_snapshot, pulled, &mut diff);
        drop(prev_snapshot);
        Prevouts::fill(state, &mut diff, resolver);
        rebuilder.tick(state, &block_template_txids, rpc.min_fee);
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
            info: self.info(),
            snapshot: rebuilder.snapshot(),
            took: started.elapsed(),
        })
    }

    fn panic_msg(payload: &(dyn Any + Send)) -> &str {
        payload
            .downcast_ref::<&'static str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("<non-string panic payload>")
    }
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use rustc_hash::FxHashMap;

    use super::*;

    #[test]
    #[should_panic(expected = "Mempool::start_with already running on this instance")]
    fn double_start_panics_with_documented_message() {
        let mempool = Mempool::for_test();
        // Simulate a prior `start_with` having grabbed the latch. We
        // can't actually call it first because the real call enters an
        // infinite loop. Flipping the atomic is what the runtime check
        // observes anyway.
        mempool.0.started.store(true, Ordering::Release);
        mempool.start_with(|_: &[(Txid, Vout)]| FxHashMap::default());
    }

    #[test]
    fn panic_msg_extracts_static_str_payload() {
        let payload = catch_unwind(|| panic!("boom static")).unwrap_err();
        assert_eq!(Mempool::panic_msg(payload.as_ref()), "boom static");
    }

    #[test]
    fn panic_msg_extracts_string_payload() {
        let payload = catch_unwind(|| panic!("boom owned {}", 42)).unwrap_err();
        assert_eq!(Mempool::panic_msg(payload.as_ref()), "boom owned 42");
    }

    #[test]
    fn panic_msg_falls_back_for_non_string_payload() {
        // Payload that isn't &str or String: the helper labels it
        // explicitly instead of dropping it on the floor.
        let payload = catch_unwind(|| std::panic::panic_any(42u32)).unwrap_err();
        assert_eq!(
            Mempool::panic_msg(payload.as_ref()),
            "<non-string panic payload>"
        );
    }
}

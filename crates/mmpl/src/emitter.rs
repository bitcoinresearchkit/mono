//! Per-cycle NDJSON emitter. Owns the cycle-over-cycle memory used to
//! turn the always-fresh `Cycle` into change-only events for `tip`,
//! `block`, and `fees`.

use std::{
    io::{self, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use brk_mempool::Cycle;
use brk_types::{Address, AddressBytes, BlockHash, NextBlockHash, RecommendedFees, Txid};
use rustc_hash::FxHashSet;
use serde_json::to_writer;

use crate::event::Event;

/// Cycle-over-cycle memory for change-event detection. `None` on the
/// first cycle, so the very first `Tip` / `Block` / `Fees` always
/// fires - downstream consumers get a baseline without a special-case
/// "current state" RPC.
///
/// `prev_block0` is `None` on cold start so the first `block` event
/// reports the entire template as `added` (one big line, then small
/// deltas).
#[derive(Default)]
pub struct Emitter {
    prev_tip_hash: Option<BlockHash>,
    prev_next_block_hash: Option<NextBlockHash>,
    prev_block0: Option<FxHashSet<Txid>>,
    prev_fees: Option<RecommendedFees>,
}

impl Emitter {
    /// Writes every event for one cycle and flushes once at the end.
    /// Per-line flushes would cost one syscall per event on busy cycles;
    /// the cycle period (~500ms) is the real "live" granularity.
    pub fn emit<W: Write>(&mut self, out: &mut W, cycle: &Cycle) -> io::Result<()> {
        let t = now_secs();
        for tx in &cycle.added {
            write_line(out, &Event::enter(t, tx))?;
        }
        for tx in &cycle.removed {
            write_line(out, &Event::leave(t, tx))?;
        }
        for bytes in &cycle.addr_enters {
            Self::emit_addr(out, t, bytes, Event::addr_enter)?;
        }
        for bytes in &cycle.addr_leaves {
            Self::emit_addr(out, t, bytes, Event::addr_leave)?;
        }
        if self.prev_tip_hash != Some(cycle.tip_hash) {
            self.prev_tip_hash = Some(cycle.tip_hash);
            write_line(out, &Event::tip(t, cycle.tip_hash, cycle.tip_height))?;
        }
        let next_block_hash = cycle.snapshot.next_block_hash;
        if self.prev_next_block_hash != Some(next_block_hash) {
            self.prev_next_block_hash = Some(next_block_hash);
            let current: FxHashSet<Txid> = cycle.snapshot.block0_txids().collect();
            let (added, removed) = match &self.prev_block0 {
                Some(prev) => (
                    current.difference(prev).copied().collect(),
                    prev.difference(&current).copied().collect(),
                ),
                None => (current.iter().copied().collect(), Vec::new()),
            };
            write_line(out, &Event::block(t, next_block_hash, added, removed))?;
            self.prev_block0 = Some(current);
        }
        if self.prev_fees.as_ref() != Some(&cycle.snapshot.fees) {
            self.prev_fees = Some(cycle.snapshot.fees.clone());
            write_line(out, &Event::fees(t, &cycle.snapshot.fees))?;
        }
        write_line(out, &Event::summary(t, cycle))?;
        out.flush()
    }

    /// Render an `AddressBytes` and emit it via `make_event`. Unrenderable
    /// bytes (e.g. exotic non-standard scripts) drop a one-line warning
    /// to stderr - the event stream stays clean for downstream `jq`.
    fn emit_addr<W: Write>(
        out: &mut W,
        t: f64,
        bytes: &AddressBytes,
        make_event: fn(f64, Address) -> Event,
    ) -> io::Result<()> {
        match Address::try_from(bytes) {
            Ok(addr) => write_line(out, &make_event(t, addr)),
            Err(e) => {
                eprintln!("mmpl: skipping addr event: {e}");
                Ok(())
            }
        }
    }
}

fn write_line<W: Write>(out: &mut W, ev: &Event) -> io::Result<()> {
    to_writer(&mut *out, ev).map_err(io::Error::other)?;
    out.write_all(b"\n")
}

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

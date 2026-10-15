//! Per-cycle NDJSON emitter. Compares each published pool with the one
//! the previous cycle saw, turning observations into change events.

use std::{
    io::{self, Write},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use brk_error::Result;
use brk_mempool::{RbfNode, ReadOnlyState};
use brk_rpc::Client;
use brk_types::{BlockHash, FeeRate, NextBlockHash, RecommendedFees, Sats, Txid, VSize};
use rustc_hash::FxHashSet;
use serde::Deserialize;
use serde_json::{from_slice, to_writer};

use crate::event::{Event, LeaveReason};

/// The previous cycle's observations. Empty on the first cycle, so the
/// very first `Tip` / `Block` / `Fees` always fires and every live tx
/// enters: downstream consumers get a baseline without a special-case
/// "current state" call.
#[derive(Default)]
pub struct Emitter {
    prev_tip: Option<BlockHash>,
    prev_next_block_hash: Option<NextBlockHash>,
    prev_fees: Option<RecommendedFees>,
    prev_txids: FxHashSet<Txid>,
}

/// The pool totals a heartbeat reports, read from the published statistics.
#[derive(Deserialize)]
struct Totals {
    count: usize,
    vsize: VSize,
    total_fee: Sats,
}

impl Emitter {
    /// Writes every event for one published pool and flushes once at the end.
    /// Per-line flushes would cost one syscall per event on busy cycles;
    /// the cycle period (~500ms) is the real "live" granularity. Every read
    /// resolves before the first line is written, and the observations only
    /// advance once all lines are out: a failed cycle is retried whole. A pool
    /// not published yet emits nothing.
    pub fn emit<W: Write>(
        &mut self,
        out: &mut W,
        client: &Client,
        state: &ReadOnlyState,
        took: Duration,
    ) -> Result<()> {
        let Some(tip) = state.tip() else {
            return Ok(());
        };
        let t = now_secs();
        let mut events = Vec::new();
        let txids: FxHashSet<Txid> = state.txids()?.copied().collect();

        let mut added = 0;
        for txid in txids.difference(&self.prev_txids) {
            let Some(published) = state.transaction(txid, &tip)? else {
                continue;
            };
            added += 1;
            events.push(Event::Enter {
                t,
                txid: *txid,
                vsize: published.vsize,
                fee: published.tx.fee,
                rate: FeeRate::from((published.tx.fee, published.vsize)),
                first_seen: published.first_seen,
            });
        }
        let mut removed = 0;
        for txid in self.prev_txids.difference(&txids) {
            let rbf = state.rbf_for_tx(txid, &tip)?;
            let reason = match rbf.root.as_ref().and_then(|root| replacer_of(root, txid)) {
                Some(by) => LeaveReason::Replaced { by },
                None => LeaveReason::Vanished,
            };
            removed += 1;
            events.push(Event::Leave {
                t,
                txid: *txid,
                reason,
            });
        }

        if self.prev_tip != Some(tip) {
            let height = client.get_block_header_info(&tip)?.height as u32;
            events.push(Event::Tip {
                t,
                hash: tip,
                height: height.into(),
            });
        }
        let next_block_hash = state.next_block_hash().ok();
        if let Some(hash) = next_block_hash
            && self.prev_next_block_hash != Some(hash)
            && let Some(block) = state.block_stats()?.first()
        {
            events.push(Event::Block {
                t,
                hash,
                count: block.tx_count,
                vsize: block.total_vsize,
                fee: block.total_fee,
            });
        }
        let fees = state.fees().ok();
        if let Some(fees) = &fees
            && self.prev_fees.as_ref() != Some(fees)
        {
            events.push(Event::fees(t, fees));
        }
        let totals: Totals = from_slice(&state.info_json()?.0)?;
        events.push(Event::Cycle {
            t,
            added,
            removed,
            count: totals.count,
            vsize: totals.vsize,
            fee: totals.total_fee,
            took_ms: took.as_millis() as u64,
        });

        for event in &events {
            write_line(out, event)?;
        }
        out.flush()?;

        self.prev_tip = Some(tip);
        if next_block_hash.is_some() {
            self.prev_next_block_hash = next_block_hash;
        }
        if fees.is_some() {
            self.prev_fees = fees;
        }
        self.prev_txids = txids;
        Ok(())
    }
}

/// The tx that directly replaced `txid` in the replacement tree under `node`.
fn replacer_of(node: &RbfNode, txid: &Txid) -> Option<Txid> {
    node.replaces.iter().find_map(|child| {
        if child.txid == *txid {
            Some(node.txid)
        } else {
            replacer_of(child, txid)
        }
    })
}

fn write_line<W: Write>(out: &mut W, ev: &Event) -> io::Result<()> {
    to_writer(&mut *out, ev).map_err(io::Error::from)?;
    out.write_all(b"\n")
}

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

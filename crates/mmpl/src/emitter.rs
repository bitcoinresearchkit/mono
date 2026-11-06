//! Per-cycle NDJSON emitter. Compares each published pool with the one
//! the previous cycle saw, turning observations into change events.

use std::{
    io::{self, Write},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use brk_error::{Error, Result};
use brk_mempool::{RbfNode, ReadOnlyState};
use brk_rpc::Client;
use brk_types::{BlockHash, FeeRate, MempoolBlockHash, PreciseRecommendedFees, Sats, Txid, VSize};
use rustc_hash::FxHashSet;
use serde::Deserialize;
use serde_json::{from_slice, to_writer};

use crate::event::{Event, LeaveReason};

/// The previous cycle's observations. Empty on the first cycle, so it emits every available
/// state event (`Tip`, `Block`, `Fees`) and every live tx enters: consumers get a baseline
/// without a special-case "current state" call.
#[derive(Default)]
pub struct Emitter {
    prev_tip: Option<BlockHash>,
    prev_mempool_block_hash: Option<MempoolBlockHash>,
    prev_fees: Option<PreciseRecommendedFees>,
    prev_txids: FxHashSet<Txid>,
    /// `prev_txids`' hash: an unchanged pool skips rebuilding and diffing the set.
    prev_txids_hash: Option<u64>,
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
    /// advance once all lines are out: a cycle whose reads fail is retried whole
    /// (a failed write ends the stream). A pool not published yet emits nothing.
    pub fn emit<W: Write>(
        &mut self,
        out: &mut W,
        client: &Client,
        state: &ReadOnlyState,
        tick: Duration,
    ) -> Result<()> {
        let Some(tip) = state.tip() else {
            return Ok(());
        };
        let t = now_secs();
        let mut events = Vec::new();
        // An unchanged pool (its txids' hash) has no one entering or leaving: the set isn't rebuilt.
        let txids_hash = state.txids_hash()?;
        let txids: Option<FxHashSet<Txid>> = if self.prev_txids_hash == Some(txids_hash) {
            None
        } else {
            Some(state.txids()?.copied().collect())
        };
        let (mut added, mut removed) = (0, 0);
        if let Some(now) = &txids {
            for txid in now.difference(&self.prev_txids) {
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
            for txid in self.prev_txids.difference(now) {
                let replacer = match state.rbf_for_tx(txid, &tip) {
                    Ok(rbf) => rbf.root.as_ref().and_then(|root| replacer_of(root, txid)),
                    // A history past the RBF traversal bounds stays unclassified: retrying the
                    // cycle would hit it again until it ages out, stalling the stream.
                    Err(Error::Internal(_)) => None,
                    Err(e) => return Err(e),
                };
                let reason = match replacer {
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
        }

        if self.prev_tip != Some(tip) {
            let height = client.get_block_header_info(&tip)?.height as u32;
            events.push(Event::Tip {
                t,
                hash: tip,
                height: height.into(),
            });
        }
        let mempool_block_hash = state
            .mempool_block_source(0)
            .ok()
            .flatten()
            .map(|block| block.hash());
        if let Some(hash) = mempool_block_hash
            && self.prev_mempool_block_hash != Some(hash)
            && let Some(block) = state.block_stats()?.first()
        {
            events.push(Event::Block {
                t,
                hash,
                count: block.tx_count,
                vsize: VSize::from(block.total_weight),
                fee: block.total_fee,
            });
        }
        let fees = state.precise_fees().ok();
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
            tick_ms: tick.as_millis() as u64,
        });

        for event in &events {
            write_line(out, event)?;
        }
        out.flush()?;

        self.prev_tip = Some(tip);
        if mempool_block_hash.is_some() {
            self.prev_mempool_block_hash = mempool_block_hash;
        }
        if fees.is_some() {
            self.prev_fees = fees;
        }
        if let Some(txids) = txids {
            self.prev_txids = txids;
            self.prev_txids_hash = Some(txids_hash);
        }
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

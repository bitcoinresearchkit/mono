//! NDJSON event schema. One [`Event`] per line; consumers pipe to
//! `jq` / `grep` to filter. Per-event fields are flat (no nested
//! objects) so `jq -c 'select(...)'` works without `..` walks.

use brk_mempool::{Cycle, TxAdded, TxRemoval, TxRemoved};
use brk_types::{
    Address, BlockHash, FeeRate, Height, NextBlockHash, RecommendedFees, Sats, Timestamp, Txid,
    VSize,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    /// A tx entered the pool this cycle (either brand new or revived
    /// from the graveyard - the stream collapses both to one event).
    Enter {
        t: f64,
        txid: Txid,
        vsize: VSize,
        fee: Sats,
        rate: FeeRate,
        first_seen: Timestamp,
    },
    /// A tx left the pool this cycle. `rate` is the package-effective
    /// rate at burial, not raw fee/vsize.
    Leave {
        t: f64,
        txid: Txid,
        #[serde(flatten)]
        reason: LeaveReason,
        rate: FeeRate,
    },
    /// An address went 0 → 1+ live mempool txs this cycle. Same-cycle
    /// flip-flops are collapsed by the upstream tracker (no event).
    AddrEnter { t: f64, addr: Address },
    /// An address went 1+ → 0 live mempool txs this cycle.
    AddrLeave { t: f64, addr: Address },
    /// New confirmed block: bitcoind's chain tip moved since the last
    /// cycle. `height` is the tip's own height (one less than the next
    /// block being templated).
    Tip {
        t: f64,
        hash: BlockHash,
        height: Height,
    },
    /// The projected next block changed (different tx set or order).
    /// `hash` is the same opaque content hash used as the mempool ETag.
    /// `added`/`removed` is the txid-level diff against the previous
    /// template; on the very first cycle `added` is the full template
    /// and `removed` is empty.
    Block {
        t: f64,
        hash: NextBlockHash,
        added: Vec<Txid>,
        removed: Vec<Txid>,
    },
    /// Recommended fee rates changed since the last cycle.
    Fees {
        t: f64,
        fastest: FeeRate,
        half_hour: FeeRate,
        hour: FeeRate,
        economy: FeeRate,
        minimum: FeeRate,
    },
    /// Per-cycle heartbeat. Always emitted, even on idle cycles, so
    /// downstream consumers see a steady pulse and can spot stalls.
    /// `addr_enters`/`addr_leaves` count the post-cancellation 0↔1+
    /// address transitions this cycle.
    Cycle {
        t: f64,
        added: usize,
        removed: usize,
        addr_enters: usize,
        addr_leaves: usize,
        count: usize,
        vsize: VSize,
        fee: Sats,
        took_ms: u64,
    },
}

#[derive(Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum LeaveReason {
    Replaced { by: Txid },
    Vanished,
}

impl Event {
    pub fn enter(t: f64, tx: &TxAdded) -> Self {
        Self::Enter {
            t,
            txid: tx.txid,
            vsize: tx.vsize,
            fee: tx.fee,
            rate: tx.fee_rate,
            first_seen: tx.first_seen,
        }
    }

    pub fn leave(t: f64, tx: &TxRemoved) -> Self {
        Self::Leave {
            t,
            txid: tx.txid,
            reason: LeaveReason::from(tx.reason),
            rate: tx.chunk_rate,
        }
    }

    pub fn addr_enter(t: f64, addr: Address) -> Self {
        Self::AddrEnter { t, addr }
    }

    pub fn addr_leave(t: f64, addr: Address) -> Self {
        Self::AddrLeave { t, addr }
    }

    pub fn tip(t: f64, hash: BlockHash, height: Height) -> Self {
        Self::Tip { t, hash, height }
    }

    pub fn block(t: f64, hash: NextBlockHash, added: Vec<Txid>, removed: Vec<Txid>) -> Self {
        Self::Block {
            t,
            hash,
            added,
            removed,
        }
    }

    pub fn fees(t: f64, fees: &RecommendedFees) -> Self {
        Self::Fees {
            t,
            fastest: fees.fastest_fee,
            half_hour: fees.half_hour_fee,
            hour: fees.hour_fee,
            economy: fees.economy_fee,
            minimum: fees.minimum_fee,
        }
    }

    pub fn summary(t: f64, cycle: &Cycle) -> Self {
        Self::Cycle {
            t,
            added: cycle.added.len(),
            removed: cycle.removed.len(),
            addr_enters: cycle.addr_enters.len(),
            addr_leaves: cycle.addr_leaves.len(),
            count: cycle.info.count,
            vsize: cycle.info.vsize,
            fee: cycle.info.total_fee,
            took_ms: cycle.took.as_millis() as u64,
        }
    }
}

impl From<TxRemoval> for LeaveReason {
    fn from(reason: TxRemoval) -> Self {
        match reason {
            TxRemoval::Replaced { by } => Self::Replaced { by },
            TxRemoval::Vanished => Self::Vanished,
        }
    }
}

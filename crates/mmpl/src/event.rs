//! NDJSON event schema. One [`Event`] per line; consumers pipe to
//! `jq` / `grep` to filter. Per-event fields are flat (no nested
//! objects) so `jq -c 'select(...)'` works without `..` walks.

use brk_types::{
    BlockHash, FeeRate, Height, NextBlockHash, RecommendedFees, Sats, Timestamp, Txid, VSize,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    /// A tx entered the published pool since the last cycle.
    Enter {
        t: f64,
        txid: Txid,
        vsize: VSize,
        fee: Sats,
        rate: FeeRate,
        first_seen: Timestamp,
    },
    /// A tx left the published pool since the last cycle.
    Leave {
        t: f64,
        txid: Txid,
        #[serde(flatten)]
        reason: LeaveReason,
    },
    /// New confirmed block: the pool's chain tip moved since the last cycle.
    Tip {
        t: f64,
        hash: BlockHash,
        height: Height,
    },
    /// The projected next block changed. `hash` is the same opaque content
    /// hash used as the block template's identity.
    Block {
        t: f64,
        hash: NextBlockHash,
        count: u32,
        vsize: VSize,
        fee: Sats,
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
    Cycle {
        t: f64,
        added: usize,
        removed: usize,
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
}

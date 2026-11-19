//! Permanent observations of the block this server expected, never reconstructed on reads.

use std::{
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::{Duration, Instant},
};

use brk_types::{BlockHash, Height, Sats, Timestamp, Txid, Vout, Weight};
use rustc_hash::FxHashMap;

use crate::{ReadOnlyMempool, ReadOnlyState};

mod compare;
mod record;
mod store;

pub use record::{Audit, Difference, Expected, Mined, Summary};
pub use store::{AuditReader, AuditStore};

/// A monotonic observation clock, separate from publication time: unchanged successful cycles
/// refresh it, failed cycles do not, and an explicit Core reload revokes it.
pub(crate) struct Observations {
    epoch: Instant,
    stamp: AtomicU64,
}

impl Default for Observations {
    fn default() -> Self {
        Self { epoch: Instant::now(), stamp: AtomicU64::new(0) }
    }
}

impl Observations {
    pub(crate) fn clear(&self) {
        self.stamp.store(0, Ordering::Release);
    }

    pub(crate) fn publish(&self, at: Instant) {
        self.stamp.store(at.duration_since(self.epoch).as_nanos() as u64 + 1, Ordering::Release);
    }
}

/// Owned pre-block publication and receipt clocks, retained across the chain update.
pub struct Capture {
    state: Arc<ReadOnlyState>,
    pub age: Duration,
    receipt: Timestamp,
}

impl ReadOnlyMempool {
    /// Pin exactly once at receipt. Reading the stamp first can only overstate the age of a
    /// publication that races this read. A withheld observation retains its original stamp.
    pub fn capture_audit(&self) -> Option<Capture> {
        self.capture_audit_at(Timestamp::now(), Instant::now())
    }

    fn capture_audit_at(&self, receipt: Timestamp, at: Instant) -> Option<Capture> {
        let stamp = self.observations.stamp.load(Ordering::Acquire);
        if stamp == 0 {
            return None;
        }
        let state = self.load();
        state.pool.as_ref()?;
        let age = at.saturating_duration_since(
            self.observations.epoch + Duration::from_nanos(stamp - 1),
        );
        (age <= Duration::from_secs(15)).then_some(Capture { state, age, receipt })
    }
}

/// Contiguous indexed non-coinbase rows and the indexed coinbase weight. The production caller
/// constructs this under one chain pin, without decoding a block or calling Core.
pub struct MinedBlock {
    pub hash: BlockHash,
    pub parent: BlockHash,
    pub height: Height,
    pub timestamp: Timestamp,
    pub weight: Weight,
    /// Includes coinbase at position zero; its fee is zero.
    pub transactions: Vec<MinedTransaction>,
    /// Input outpoint -> zero-based position of its mined spender (coinbase excluded).
    pub spends: FxHashMap<(Txid, Vout), u16>,
}

pub struct MinedTransaction {
    pub txid: Txid,
    pub weight: Weight,
    pub fee: Sats,
}

impl Capture {
    fn anchor(&self) -> BlockHash {
        self.state.tip().expect("a capture has a loaded pool")
    }

    /// A later chain publication cannot supply a missing or differently anchored capture.
    pub fn compare(self, block: &MinedBlock) -> Option<Audit> {
        (self.anchor() == block.parent).then(|| compare::compare(&self, block))
    }
}

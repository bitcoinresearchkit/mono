//! Fee reads: tier recommendations, projected-block stats, per-tx rates.

use brk_types::{FeeRate, RecommendedFees, Txid};

use crate::{Mempool, snapshot::BlockStats};

impl Mempool {
    #[must_use]
    pub fn fees(&self) -> RecommendedFees {
        self.snapshot().fees.clone()
    }

    #[must_use]
    pub fn block_stats(&self) -> Vec<BlockStats> {
        self.snapshot().block_stats.clone()
    }

    /// Effective fee rate for a live tx: snapshot's linearized chunk
    /// rate. Falls back to `fee/vsize` for txs added since the latest
    /// snapshot was built (apply -> same-cycle tick gap).
    pub fn live_effective_fee_rate(&self, txid: &Txid) -> Option<FeeRate> {
        if let Some(rate) = self.snapshot().chunk_rate_for(txid) {
            return Some(rate);
        }
        self.read().txs.entry(txid).map(|entry| entry.fee_rate())
    }

    /// Linearized chunk rate captured at burial - same value
    /// `live_effective_fee_rate` returned while the tx was alive, so an
    /// evicted RBF predecessor reports the package-effective rate it
    /// had in the mempool, not a misleading isolated `fee/vsize`.
    pub fn graveyard_fee_rate(&self, txid: &Txid) -> Option<FeeRate> {
        self.read().graveyard.get(txid).map(|tomb| tomb.chunk_rate)
    }
}

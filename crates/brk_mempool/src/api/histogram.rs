//! ReadOnlyState info + price-blending output histogram.

use std::sync::Arc;

use brk_error::Result;
use brk_oracle::HistogramRaw;
use brk_types::BlockHash;

use crate::ReadOnlyState;

impl ReadOnlyState {
    /// Last complete membership statistics as JSON, with a hash of those bytes, built once per
    /// publication. Live updates and unresolved inputs do not hide this publication; only
    /// startup has no statistics yet.
    pub fn info_json(&self) -> Result<(Arc<[u8]>, u64)> {
        Ok(self.pool()?.info_json.clone())
    }

    /// Snapshot of pre-bucketed round-dollar-eligible bins across all live
    /// mempool tx outputs. Maintained incrementally by `TxStore` on every
    /// insert/remove, so this hot path is `O(NUM_BINS)` regardless of pool
    /// size. Used by `live_price` to blend the mempool into the committed
    /// oracle without re-parsing scripts per request. Requires a completed
    /// publication at the requested confirmed-chain tip.
    pub fn live_eligible_histogram(&self, tip: &BlockHash) -> Result<HistogramRaw> {
        let state = self.pool()?.at(tip)?;
        Ok(state.txs.live_eligible_histogram())
    }

    /// Snapshot of the raw histogram: live mempool outputs binned by value
    /// (zero and out-of-range values dropped), no payment filtering. Requires
    /// a completed publication at the requested tip.
    pub fn live_raw_histogram(&self, tip: &BlockHash) -> Result<HistogramRaw> {
        let state = self.pool()?.at(tip)?;
        Ok(state.txs.live_raw_histogram())
    }
}

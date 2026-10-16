//! Fee reads: tier recommendations, projected-block stats, per-tx rates.

use brk_error::Result;
use brk_types::{PreciseRecommendedFees, RecommendedFees};

use crate::{ReadOnlyState, snapshot::BlockStats};

impl ReadOnlyState {
    /// mempool.space's whole-sat/vB recommendations (`/api/v1/fees/recommended`).
    pub fn fees(&self) -> Result<RecommendedFees> {
        let snapshot = self.snapshot();
        snapshot.ensure_projection()?;
        Ok(snapshot.fees.clone())
    }

    /// mempool.space's precise recommendations (`/api/v1/fees/precise`).
    pub fn precise_fees(&self) -> Result<PreciseRecommendedFees> {
        let snapshot = self.snapshot();
        snapshot.ensure_projection()?;
        Ok(snapshot.precise_fees.clone())
    }

    pub fn block_stats(&self) -> Result<Vec<BlockStats>> {
        let snapshot = self.snapshot();
        snapshot.ensure_projection()?;
        Ok(snapshot.block_stats.clone())
    }
}

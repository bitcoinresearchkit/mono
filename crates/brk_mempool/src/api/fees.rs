use brk_error::Result;
use brk_types::PreciseRecommendedFees;

use crate::{JsonBody, ReadOnlyState, snapshot::BlockStats};

impl ReadOnlyState {
    /// mempool.space's whole-sat/vB recommendations (`/api/v1/fees/recommended`), as JSON.
    pub fn fees_json(&self) -> Result<JsonBody> {
        self.snapshot().fees_json()
    }

    /// mempool.space's precise recommendations (`/api/v1/fees/precise`).
    pub fn precise_fees(&self) -> Result<PreciseRecommendedFees> {
        let snapshot = self.snapshot();
        snapshot.ensure_fees()?;
        Ok(snapshot.precise_fees.clone())
    }

    /// [`Self::precise_fees`] as JSON.
    pub fn precise_fees_json(&self) -> Result<JsonBody> {
        self.snapshot().precise_fees_json()
    }

    pub fn block_stats(&self) -> Result<Vec<BlockStats>> {
        let snapshot = self.snapshot();
        snapshot.ensure_projection()?;
        Ok(snapshot.block_stats.clone())
    }

    /// Every projected block's stats (`/api/v1/fees/mempool-blocks`), as JSON.
    pub fn mempool_blocks_json(&self) -> Result<JsonBody> {
        self.snapshot().blocks_json()
    }
}

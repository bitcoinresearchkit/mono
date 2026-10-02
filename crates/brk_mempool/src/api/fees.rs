//! Fee reads: tier recommendations, projected-block stats, per-tx rates.

use brk_error::Result;
use brk_types::RecommendedFees;

use crate::{ReadOnlyState, snapshot::BlockStats};

impl ReadOnlyState {
    pub fn fees(&self) -> Result<RecommendedFees> {
        let snapshot = self.snapshot();
        snapshot.ensure_projection()?;
        Ok(snapshot.fees.clone())
    }

    pub fn block_stats(&self) -> Result<Vec<BlockStats>> {
        let snapshot = self.snapshot();
        snapshot.ensure_projection()?;
        Ok(snapshot.block_stats.clone())
    }
}

use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::Version;
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::indexes;

use super::{AddrCountsVecs, AddrTypeToAddrCount};

/// Paired funded + cumulative-total address counts, used by exposed, reused,
/// and respent. On-disk naming: `"{name}_addr_count"` (funded) and
/// `"total_{name}_addr_count"` (total).
#[derive(Traversable)]
pub struct AddrCountFundedTotalVecs<M: StorageMode = Rw> {
    pub funded: AddrCountsVecs<M>,
    pub total: AddrCountsVecs<M>,
}

impl AddrCountFundedTotalVecs {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        Ok(Self {
            funded: AddrCountsVecs::forced_import(
                db,
                &format!("{name}_addr_count"),
                version,
                indexes,
            )?,
            total: AddrCountsVecs::forced_import(
                db,
                &format!("total_{name}_addr_count"),
                version,
                indexes,
            )?,
        })
    }

    pub fn min_stateful_len(&self) -> usize {
        self.funded
            .min_stateful_len()
            .min(self.total.min_stateful_len())
    }

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.funded
            .par_iter_height_mut()
            .chain(self.total.par_iter_height_mut())
    }

    pub fn reset_height(&mut self) -> Result<()> {
        self.funded.reset_height()?;
        self.total.reset_height()?;
        Ok(())
    }

    #[inline(always)]
    pub fn push_counts(&mut self, funded: &AddrTypeToAddrCount, total: &AddrTypeToAddrCount) {
        self.funded.push_counts(funded);
        self.total.push_counts(total);
    }
}

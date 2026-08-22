use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::Version;
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::{AddrCountsVecs, AddrTypeToAddrCount};

/// Paired funded + cumulative-total address counts, used by exposed, reused,
/// and respent. On-disk naming: `"{name}_addr_count"` (funded) and
/// `"total_{name}_addr_count"` (total).
#[derive(Traversable)]
pub struct AddrCountFundedTotalVecs<M: StorageMode = Rw> {
    /// Number of addresses that hold unspent outputs at the represented block
    /// and satisfy an address predicate.
    pub funded: AddrCountsVecs<M>,
    /// Number of addresses that have ever satisfied an address
    /// predicate, whether or not they hold unspent outputs at the represented
    /// block.
    pub total: AddrCountsVecs<M>,
}

impl AddrCountFundedTotalVecs {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> Result<Self> {
        Ok(Self {
            funded: AddrCountsVecs::forced_import(
                db,
                &format!("{name}_addr_count"),
                version,
                mappings,
            )?,
            total: AddrCountsVecs::forced_import(
                db,
                &format!("total_{name}_addr_count"),
                version,
                mappings,
            )?,
        })
    }

    pub fn min_resume_len(&self) -> usize {
        self.funded
            .min_resume_len()
            .min(self.total.min_resume_len())
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

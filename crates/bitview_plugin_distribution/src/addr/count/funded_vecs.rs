use brk_error::Result;

use bitview_cohort::{AmountRange, CohortContext};
use bitview_traversable::Traversable;
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::metrics::ColumnarAmount;
use bitview_compute::{CachedWindowStartVec, LazyPerBlockWithDeltas, Windows};

use super::{AddrCountsVecs, AddrTypeToAddrCount};

#[derive(Traversable)]
pub struct FundedAddrCountsVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub counts: AddrCountsVecs<M>,
    /// Number of funded addresses grouped by balance at the represented block.
    pub balance: ColumnarAmount<
        StoredU64,
        LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>,
        M,
    >,
}

impl FundedAddrCountsVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            counts: AddrCountsVecs::forced_import(db, "addr_count", version, mappings)?,
            balance: ColumnarAmount::forced_import(
                db,
                "addrs_addr_count_by_balance_range",
                CohortContext::Addr,
                "addr_count",
                version + Version::ONE,
                |name, source| {
                    LazyPerBlockWithDeltas::from_boxed_height_source(
                        name,
                        version + Version::ONE,
                        source,
                        Version::TWO,
                        mappings,
                        cached_starts,
                    )
                },
            )?,
        })
    }

    pub fn min_resume_len(&self) -> usize {
        self.counts.min_resume_len().min(self.balance.len())
    }

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.counts
            .par_iter_height_mut()
            .chain(rayon::iter::once(self.balance.stored_mut()))
    }

    pub fn reset_height(&mut self) -> Result<()> {
        self.counts.reset_height()?;
        self.balance.reset()?;
        Ok(())
    }

    #[inline(always)]
    pub fn push_counts(&mut self, counts: &AddrTypeToAddrCount) {
        self.counts.push_counts(counts);
    }

    #[inline(always)]
    pub fn push_balance(&mut self, counts: AmountRange<StoredU64>) {
        self.balance.push(counts);
    }
}

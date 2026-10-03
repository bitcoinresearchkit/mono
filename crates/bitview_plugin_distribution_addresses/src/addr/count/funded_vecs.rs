use bitview_cohort::{AmountRange, CohortContext};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockWithDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::{AddrCountsVecs, AddrTypeToAddrCount};
use bitview_vecs::AmountSources;

#[derive(Traversable)]
pub struct FundedAddrCountsVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub counts: AddrCountsVecs<M>,
    /// Number of funded addresses grouped by balance at the represented block.
    pub balance: AmountSources<
        StoredU64,
        LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>,
        M,
    >,
}

impl FundedAddrCountsVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            counts: AddrCountsVecs::forced_import(db, "addr_count", version, mappings)?,
            balance: AmountSources::forced_import(
                db,
                "addrs_addr_count_by_balance_range",
                CohortContext::Addr,
                "addr_count",
                version + Version::ONE,
                |name, source| {
                    LazyPerBlockWithDeltas::from_height_source(
                        name,
                        version + Version::ONE,
                        source,
                        Version::TWO,
                        mappings,
                        window_starts,
                    )
                },
            )?,
        })
    }

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.counts.par_iter_height_mut().chain(
            self.balance
                .stored_vecs_mut()
                .collect::<Vec<_>>()
                .into_par_iter(),
        )
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

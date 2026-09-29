use crate::addr_groups::SizeAndAddrGroups;
use crate::cumulative_value::CumulativeSizeValueSources;
use crate::groups::SizeGroups;
use crate::values::SizeValues;
use bitview_cohort::{AmountRange, CohortContext};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyValuePerBlockCumulativeRolling, LazyWindowStartVec, SatsCents};
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::metrics::AmountValueSources;

#[derive(Traversable)]
pub struct CumulativeValueByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// UTXO groups and spent output value grouped by the spending address's
    /// balance immediately before the spend.
    pub cohorts: SizeAndAddrGroups<
        LazyValuePerBlockCumulativeRolling,
        AmountValueSources<LazyValuePerBlockCumulativeRolling, M>,
    >,
    #[traversable(hidden)]
    pub stored: CumulativeSizeValueSources<M>,
}

impl CumulativeValueByCohort {
    pub fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = CumulativeSizeValueSources::forced_import(
            db,
            &format!("{metric}_cumulative"),
            version,
        )?;
        let cohorts = SizeGroups::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let SatsCents { sats, cents } = stored
                .sources(cohort_id, &name, version)
                .expect("supported stored value cohort");
            LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                &name,
                version,
                &sats,
                &cents,
                mappings,
                window_starts,
            )
        });
        let addr_version = version + Version::ONE;
        let addr_balance = AmountValueSources::forced_import(
            db,
            &format!("addrs_{metric}_cumulative_by_balance_range"),
            CohortContext::Addr,
            metric,
            addr_version,
            |name, sats, cents| {
                LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                    name,
                    addr_version,
                    &sats,
                    &cents,
                    mappings,
                    window_starts,
                )
            },
        )?;
        Ok(Self {
            cohorts: SizeAndAddrGroups {
                utxo: cohorts,
                addr_balance,
            },
            stored,
        })
    }

    #[inline(always)]
    pub fn push_block(&mut self, sats: SizeValues<Sats>, cents: SizeValues<Cents>) {
        self.stored.push_block(sats, cents);
    }

    #[inline(always)]
    pub fn push_addr_balance(&mut self, sats: &AmountRange<Sats>, cents: &AmountRange<Cents>) {
        self.cohorts.addr_balance.push_cumulative(sats, cents);
    }

    pub fn min_len(&self) -> usize {
        self.stored.min_len().min(self.cohorts.addr_balance.len())
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .stored_vecs_mut()
            .chain(self.cohorts.addr_balance.stored_vecs_mut())
    }
}

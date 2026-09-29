use crate::addr_groups::SizeAndAddrGroups;
use crate::groups::SizeGroups;
use crate::sources::SizeSources;
use crate::values::SizeValues;
use bitview_cohort::{AmountRange, CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazySpotValuePerBlock;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, ReadableCloneableVec, Rw, StorageMode};

use crate::metrics::AmountSources;

#[derive(Traversable)]
pub struct SupplyTotal<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts:
        SizeAndAddrGroups<LazySpotValuePerBlock, AmountSources<Sats, LazySpotValuePerBlock, M>>,
    #[traversable(hidden)]
    pub stored: SizeSources<Sats, M>,
    #[traversable(skip)]
    all_supply: ReadableBoxedVec<Height, Sats>,
}

impl SupplyTotal {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let stored = SizeSources::forced_import(db, "supply_sats", version)?;
        let all_supply = all_supply.read_only_boxed_clone();
        let cohorts = SizeGroups::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "supply");
            let source = stored.get(cohort_id).expect("size supply source");
            LazySpotValuePerBlock::from_sats_source(&name, version, source, mappings, spot_price)
        });
        let addr_balance = AmountSources::forced_import(
            db,
            "addrs_supply_sats_by_balance_range",
            CohortContext::Addr,
            "supply",
            version + Version::ONE,
            |name, source| {
                LazySpotValuePerBlock::from_sats_source(
                    name,
                    version + Version::ONE,
                    source,
                    mappings,
                    spot_price,
                )
            },
        )?;

        Ok(Self {
            cohorts: SizeAndAddrGroups {
                utxo: cohorts,
                addr_balance,
            },
            stored,
            all_supply,
        })
    }

    pub fn min_len(&self) -> usize {
        self.stored.min_len().min(self.cohorts.addr_balance.len())
    }

    pub fn get(&self, cohort_id: CohortId) -> Option<&LazySpotValuePerBlock> {
        self.cohorts.utxo.get(cohort_id)
    }

    pub fn all_supply(&self) -> &ReadableBoxedVec<Height, Sats> {
        &self.all_supply
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: SizeValues<Sats>) {
        self.stored.push(cohort_values);
    }

    #[inline(always)]
    pub fn push_addr_balance(&mut self, values: AmountRange<Sats>) {
        self.cohorts.addr_balance.push(values);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .stored_vecs_mut()
            .chain(self.cohorts.addr_balance.stored_vecs_mut())
    }
}

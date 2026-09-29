use super::{SupplyBase, SupplyTotal};
use crate::addr_groups::SizeAndAddrGroups;
use bitview_cohort::{AmountRange, CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPercentPerBlock, LazyRollingDeltasAmountFromHeight, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{
    Cents, Height, PartsPerMillion32, PartsPerMillionSigned64, Sats, SatsSigned, Version,
};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};
#[derive(Traversable)]
pub struct SupplyVecs<M: StorageMode = Rw> {
    pub total: SupplyTotal<M>,
    pub delta: SizeAndAddrGroups<
        LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
    >,
    pub dominance: SizeAndAddrGroups<LazyPercentPerBlock<PartsPerMillion32>>,
}
impl SupplyVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Box<Self>> {
        let total = SupplyTotal::forced_import(db, version, mappings, spot_price, all_supply)?;
        let all_supply = total.all_supply();
        let utxo = total.cohorts.utxo.map_with_id(|cohort_id, total| {
            if matches!(cohort_id, CohortId::All) {
                SupplyBase::from_all_total(version, total.clone(), mappings, window_starts)
            } else {
                SupplyBase::from_total(
                    CohortContext::Utxo,
                    cohort_id,
                    version,
                    total.clone(),
                    all_supply,
                    mappings,
                    window_starts,
                )
            }
        });
        let addr_balance = AmountRange::from_fn(|id| {
            let cohort_id = id.cohort();
            let total = id.select(&total.cohorts.addr_balance.series);
            SupplyBase::from_total(
                CohortContext::Addr,
                cohort_id,
                version + Version::ONE,
                total.clone(),
                all_supply,
                mappings,
                window_starts,
            )
        });
        let bases = SizeAndAddrGroups { utxo, addr_balance };
        let delta = bases.map_with_id(|_, _, base| base.delta.clone());
        let dominance = bases.map_with_id(|_, _, base| base.dominance.clone());
        Ok(Box::new(Self {
            total,
            delta,
            dominance,
        }))
    }
    pub fn min_resume_len(&self) -> usize {
        self.total.min_len()
    }
    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.total.stored_vecs_mut()
    }
}

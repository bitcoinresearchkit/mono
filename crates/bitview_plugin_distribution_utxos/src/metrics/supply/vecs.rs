use super::{SupplyBase, SupplyTotal};
use bitview_cohort::CohortContext;
use bitview_cohort::UtxoGroups;
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
    pub delta:
        UtxoGroups<LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>>,
    pub dominance: UtxoGroups<LazyPercentPerBlock<PartsPerMillion32>>,
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
        let utxo = total.cohorts.map_with_id(|cohort_id, total| {
            SupplyBase::from_total(
                CohortContext::Utxo,
                cohort_id,
                version,
                total.clone(),
                all_supply,
                mappings,
                window_starts,
            )
        });
        let delta = utxo.map_with_id(|_, base| base.delta.clone());
        let dominance = utxo.map_with_id(|_, base| base.dominance.clone());
        Ok(Box::new(Self {
            total,
            delta,
            dominance,
        }))
    }
    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.total.stored_vecs_mut()
    }
}

use super::CumulativeValueByCohort;
use bitview_cohort::AmountRange;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::BinaryTransform;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};
#[derive(Traversable)]
pub struct ActivityVecs<M: StorageMode = Rw> {
    pub transfer_volume: Box<CumulativeValueByCohort<M>>,
}
impl ActivityVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            transfer_volume: Box::new(CumulativeValueByCohort::forced_import(
                db,
                "transfer_volume",
                version + Version::ONE,
                mappings,
                windows,
            )?),
        }))
    }
    pub fn push_addr_balance(&mut self, price: Cents, values: &AmountRange<Sats>) {
        let cents = AmountRange::from_fn(|id| SatsToCents::apply(*id.select(values), price));
        self.transfer_volume.push_addr_balance(values, &cents);
    }
    pub fn min_resume_len(&self) -> usize {
        self.transfer_volume.min_len()
    }
    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.transfer_volume.stored_vecs_mut()
    }
}

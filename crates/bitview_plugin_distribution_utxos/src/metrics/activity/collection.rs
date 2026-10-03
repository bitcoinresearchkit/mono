use super::CumulativeValueByCohort;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::Version;
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

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.transfer_volume.stored_vecs_mut()
    }
}

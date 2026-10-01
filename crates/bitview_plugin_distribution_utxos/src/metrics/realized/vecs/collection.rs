use super::{CumulativeRealizedByCohort, RealizedCapByCohort, RealizedPriceByCohort};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};
#[derive(Traversable)]
pub struct RealizedVecs<M: StorageMode = Rw> {
    pub cap: RealizedCapByCohort<M>,
    pub price: RealizedPriceByCohort<M>,
    pub profit: CumulativeRealizedByCohort<M>,
    pub loss: CumulativeRealizedByCohort<M>,
}
impl RealizedVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Box<Self>> {
        let cap = RealizedCapByCohort::forced_import(db, version, mappings)?;
        let price = RealizedPriceByCohort::forced_import(db, version, mappings)?;
        let profit = CumulativeRealizedByCohort::forced_import(
            db,
            "realized_profit",
            version + Version::ONE,
            mappings,
            window_starts,
        )?;
        let loss = CumulativeRealizedByCohort::forced_import(
            db,
            "realized_loss",
            version + Version::ONE,
            mappings,
            window_starts,
        )?;
        Ok(Box::new(Self {
            cap,
            price,
            profit,
            loss,
        }))
    }

    pub fn min_resume_len(&self) -> usize {
        self.cap
            .stored
            .min_len()
            .min(self.price.stored.min_len())
            .min(self.profit.stored.min_len())
            .min(self.loss.stored.min_len())
    }
    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.cap
            .stored
            .stored_vecs_mut()
            .chain(self.price.stored.stored_vecs_mut())
            .chain(self.profit.stored.stored_vecs_mut())
            .chain(self.loss.stored.stored_vecs_mut())
    }
}

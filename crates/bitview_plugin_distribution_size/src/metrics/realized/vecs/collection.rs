use super::{CumulativeRealizedByCohort, RealizedCapByCohort, RealizedPriceByCohort};
use crate::groups::SizeGroups;
use bitview_cohort::{AmountRange, CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::MvrvToNupl;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, LazyRatioPerBlock, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Cents, Height, PartsPerMillionSigned32, PriceRatio, StoredF32, Version};
use vecdb::{AnyStoredVec, Database, Ident, ReadableBoxedVec, Rw, StorageMode};
#[derive(Traversable)]
pub struct RealizedVecs<M: StorageMode = Rw> {
    pub nupl: SizeGroups<LazyRatioPerBlock<PartsPerMillionSigned32, PriceRatio>>,
    pub cap: RealizedCapByCohort<M>,
    pub price: RealizedPriceByCohort<M>,
    pub profit: CumulativeRealizedByCohort<M>,
    pub loss: CumulativeRealizedByCohort<M>,
    pub mvrv: SizeGroups<LazyPerBlock<StoredF32>>,
}
impl RealizedVecs {
    fn cohort_version(version: Version, cohort_id: CohortId) -> Version {
        version
            + if matches!(cohort_id, CohortId::All) {
                Version::ONE
            } else {
                Version::ZERO
            }
    }
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Box<Self>> {
        let cap = RealizedCapByCohort::forced_import(db, version, mappings, window_starts)?;
        let price = RealizedPriceByCohort::forced_import(db, version, mappings, spot_price)?;
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
        let nupl = price.cohorts.map_with_id(|id, price| {
            LazyRatioPerBlock::from_lazy_source::<MvrvToNupl, PriceRatio>(
                &CohortContext::Utxo.metric_name(id, "nupl"),
                version + Version::new(5),
                &price.relative.ppm,
            )
        });
        let mvrv = price.cohorts.map_with_id(|cohort_id, price| {
            LazyPerBlock::from_lazy::<Ident, PriceRatio>(
                &CohortContext::Utxo.metric_name(cohort_id, "mvrv"),
                Self::cohort_version(version, cohort_id),
                &price.relative.ratio,
            )
        });
        Ok(Box::new(Self {
            cap,
            price,
            profit,
            loss,
            mvrv,
            nupl,
        }))
    }
    pub fn push_addr_balance(
        &mut self,
        cap: AmountRange<Cents>,
        profit: &AmountRange<Cents>,
        loss: &AmountRange<Cents>,
    ) {
        self.cap.cohorts.addr_balance.push(cap);
        self.profit.cohorts.addr_balance.push_cumulative(profit);
        self.loss.cohorts.addr_balance.push_cumulative(loss);
    }
    pub fn min_resume_len(&self) -> usize {
        self.cap
            .stored
            .min_len()
            .min(self.price.stored.min_len())
            .min(self.profit.stored.min_len())
            .min(self.loss.stored.min_len())
            .min(self.cap.cohorts.addr_balance.len())
            .min(self.profit.cohorts.addr_balance.len())
            .min(self.loss.cohorts.addr_balance.len())
    }
    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.cap
            .stored
            .stored_vecs_mut()
            .chain(self.cap.cohorts.addr_balance.stored_vecs_mut())
            .chain(self.price.stored.stored_vecs_mut())
            .chain(self.profit.stored.stored_vecs_mut())
            .chain(self.profit.cohorts.addr_balance.stored_vecs_mut())
            .chain(self.loss.stored.stored_vecs_mut())
            .chain(self.loss.cohorts.addr_balance.stored_vecs_mut())
    }
}

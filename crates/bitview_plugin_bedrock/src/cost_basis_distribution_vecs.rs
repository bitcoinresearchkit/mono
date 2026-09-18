use bitview_traversable::Traversable;
use bitview_vecs::DailyMappings;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::{CostBasisData, DailyPercentilesVecs, WeightedPair};

#[derive(Traversable)]
pub struct CostBasisDistributionVecs<M: StorageMode = Rw> {
    pub per_coin: WeightedPair<DailyPercentilesVecs<M>>,
    pub per_dollar: WeightedPair<DailyPercentilesVecs<M>>,
}

impl CostBasisDistributionVecs {
    fn import_weighting(
        db: &Database,
        cohort: &str,
        weighting: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<WeightedPair<DailyPercentilesVecs>> {
        WeightedPair::try_from_fn(|weight| {
            DailyPercentilesVecs::forced_import(
                db,
                &format!("bedrock_{}{cohort}_cost_basis_{weighting}", weight.as_str()),
                version,
                mappings,
            )
        })
    }

    pub fn forced_import(
        db: &Database,
        cohort: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        Ok(Self {
            per_coin: Self::import_weighting(db, cohort, "per_coin", version, mappings)?,
            per_dollar: Self::import_weighting(db, cohort, "per_dollar", version, mappings)?,
        })
    }

    pub fn push(&mut self, data: &WeightedPair<CostBasisData>) {
        for (target, data) in self.per_coin.iter_mut().zip(data.iter()) {
            target.push(&data.prices.per_coin);
        }
        for (target, data) in self.per_dollar.iter_mut().zip(data.iter()) {
            target.push(&data.prices.per_dollar);
        }
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.per_coin
            .iter_mut()
            .chain(self.per_dollar.iter_mut())
            .flat_map(DailyPercentilesVecs::collect_vecs_mut)
    }
}

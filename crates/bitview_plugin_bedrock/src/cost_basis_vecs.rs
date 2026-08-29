use bitview_compute::DailyMappings;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{CostBasisPercentilePrices, Version};
use vecdb::{AnyStoredVec, Database, Rw};

use crate::{DailyPercentilesVecs, WeightedPair};

#[derive(Traversable)]
pub struct CostBasisVecs<M: vecdb::StorageMode = Rw> {
    pub per_coin: WeightedPair<DailyPercentilesVecs<M>>,
    pub per_dollar: WeightedPair<DailyPercentilesVecs<M>>,
}

impl CostBasisVecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        Ok(Self {
            per_coin: Self::import_weighting(db, "per_coin", version, mappings)?,
            per_dollar: Self::import_weighting(db, "per_dollar", version, mappings)?,
        })
    }

    pub(crate) fn push(&mut self, prices: &WeightedPair<CostBasisPercentilePrices>) {
        self.per_coin.cointime.push(&prices.cointime.per_coin);
        self.per_coin.coinflow.push(&prices.coinflow.per_coin);
        self.per_dollar.cointime.push(&prices.cointime.per_dollar);
        self.per_dollar.coinflow.push(&prices.coinflow.per_dollar);
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.per_coin
            .iter_mut()
            .chain(self.per_dollar.iter_mut())
            .map(|percentiles| percentiles.stored_mut())
    }

    pub(crate) fn minimum_len(&mut self) -> usize {
        self.stored_vecs_mut()
            .map(|vec| vec.len())
            .min()
            .unwrap_or_default()
    }

    fn import_weighting(
        db: &Database,
        weighting: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<WeightedPair<DailyPercentilesVecs>> {
        WeightedPair::try_from_fn(|weight| {
            DailyPercentilesVecs::forced_import(
                db,
                &format!("bedrock_{}_cost_basis_{weighting}", weight.as_str()),
                version,
                mappings,
            )
        })
    }
}

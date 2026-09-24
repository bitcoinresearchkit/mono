use bitview_traversable::Traversable;
use bitview_vecs::{DailyMappings, DailyPercentilesVecs};
use brk_error::Result;
use brk_types::{CostBasisByPercentile, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct CostBasisMetrics<M: StorageMode = Rw> {
    pub per_coin: DailyPercentilesVecs<M>,
    pub per_dollar: DailyPercentilesVecs<M>,
}

impl CostBasisMetrics {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        Ok(Self {
            per_coin: DailyPercentilesVecs::forced_import(
                db,
                &format!("{name}_per_coin"),
                version,
                mappings,
            )?,
            per_dollar: DailyPercentilesVecs::forced_import(
                db,
                &format!("{name}_per_dollar"),
                version,
                mappings,
            )?,
        })
    }
    pub fn push(&mut self, data: &CostBasisByPercentile) {
        self.per_coin.push(&data.per_coin);
        self.per_dollar.push(&data.per_dollar);
    }
    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.per_coin
            .collect_vecs_mut()
            .into_iter()
            .chain(self.per_dollar.collect_vecs_mut())
    }
}

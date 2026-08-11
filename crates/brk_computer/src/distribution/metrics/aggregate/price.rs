use brk_cohort::{
    CohortContext, UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES, UTXOAggregate, UTXOAggregateId,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, AnyVec, CachedBoxedVec, Database, Rw, StorageMode};

use crate::{
    indexes,
    internal::{ColumnarPerBlock, LazyColumnPriceWithRatioPerBlock},
};

#[derive(Deref, DerefMut, Traversable)]
pub struct AggregatePriceWithRatioPerBlock<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub values: ColumnarPerBlock<
        Cents,
        UTXOAggregateId,
        UTXOAggregate<LazyColumnPriceWithRatioPerBlock<UTXOAggregateId>>,
        M,
    >,
}

impl AggregatePriceWithRatioPerBlock {
    pub fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let values = ColumnarPerBlock::forced_import(
            db,
            &format!("{metric}_cents_by_aggregate"),
            version,
            |source| {
                UTXOAggregate::from_fn(|id| {
                    let name = CohortContext::Utxo.metric_name(
                        id.select(&UTXO_AGGREGATE_FILTERS),
                        id.select(&UTXO_AGGREGATE_NAMES).id,
                        metric,
                    );
                    LazyColumnPriceWithRatioPerBlock::new(
                        &name, version, source, id, indexes, spot_price,
                    )
                })
            },
        )?;
        Ok(Self { values })
    }

    #[inline(always)]
    pub fn push(&mut self, row: UTXOAggregate<Cents>) {
        self.values.push(row);
    }

    pub fn len(&self) -> usize {
        self.values.height.len()
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.values.stored_mut()
    }
}

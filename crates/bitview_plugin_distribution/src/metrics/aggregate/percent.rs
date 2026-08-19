use brk_error::Result;

use bitview_traversable::Traversable;
use brk_cohort::{
    CohortContext, UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES, UTXOAggregate, UTXOAggregateId,
};
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Database, Exit, ReadableVec, Rw, StorageMode, VecValue};

use bitview_compute::{ColumnarPerBlock, FixedRatio, LazyColumnPercentPerBlock};

#[derive(Deref, DerefMut, Traversable)]
pub struct AggregatePercentPerBlock<B: FixedRatio, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub values: ColumnarPerBlock<
        B,
        UTXOAggregateId,
        UTXOAggregate<LazyColumnPercentPerBlock<B, UTXOAggregateId>>,
        M,
    >,
}

impl<B: FixedRatio> AggregatePercentPerBlock<B> {
    pub fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> Result<Self> {
        let values = ColumnarPerBlock::forced_import(
            db,
            &format!("{metric}_{}_by_aggregate", B::SUFFIX),
            version,
            |source| {
                UTXOAggregate::from_fn(|id| {
                    let name = CohortContext::Utxo.metric_name(
                        id.select(&UTXO_AGGREGATE_FILTERS),
                        id.select(&UTXO_AGGREGATE_NAMES).id,
                        metric,
                    );
                    LazyColumnPercentPerBlock::new(&name, version, source, id, indexes)
                })
            },
        )?;
        Ok(Self { values })
    }

    pub fn compute_columns2<'a, A, C, V1, V2>(
        &mut self,
        max_from: Height,
        source1: impl Fn(UTXOAggregateId) -> &'a V1,
        source2: impl Fn(UTXOAggregateId) -> &'a V2,
        transform: impl FnMut(UTXOAggregateId, A, C) -> B,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        C: VecValue,
        V1: ReadableVec<Height, A> + 'a,
        V2: ReadableVec<Height, C> + 'a,
    {
        self.values
            .compute_columns2(max_from, source1, source2, transform, exit)
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.values.stored_mut()
    }
}

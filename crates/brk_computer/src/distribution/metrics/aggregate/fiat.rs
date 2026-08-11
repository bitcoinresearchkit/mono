use brk_cohort::{UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES, UTXOAggregate, UTXOAggregateId};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, Database, ReadableCloneableVec, ReadableColumnarVec, Rw, StorageMode,
};

use crate::{
    indexes,
    internal::{ColumnarPerBlock, FiatType, LazyFiatPerBlock},
};

use super::utxo_metric_name;

#[derive(Deref, DerefMut, Traversable)]
pub struct AggregateFiatPerBlock<C: FiatType, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub values: ColumnarPerBlock<C, UTXOAggregateId, UTXOAggregate<LazyFiatPerBlock<C>>, M>,
}

impl<C: FiatType> AggregateFiatPerBlock<C> {
    pub(crate) fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let values = ColumnarPerBlock::forced_import(
            db,
            &format!("{metric}_cents_by_aggregate"),
            version,
            |source| {
                UTXOAggregate::from_fn(|id| {
                    let name = utxo_metric_name(
                        id.select(&UTXO_AGGREGATE_FILTERS),
                        id.select(&UTXO_AGGREGATE_NAMES).id,
                        metric,
                    );
                    LazyFiatPerBlock::from_boxed_cents_source(
                        &name,
                        version,
                        source
                            .column(&format!("{name}_cents"), version, id)
                            .read_only_boxed_clone(),
                        indexes,
                    )
                })
            },
        )?;
        Ok(Self { values })
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, row: UTXOAggregate<C>) {
        self.values.push(row);
    }

    pub(crate) fn len(&self) -> usize {
        self.values.height.len()
    }

    pub(crate) fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.values.stored_mut()
    }
}

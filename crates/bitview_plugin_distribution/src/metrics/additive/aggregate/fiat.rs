use brk_error::Result;

use bitview_cohort::{
    ByTerm, CohortContext, TermId, UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES, UTXOAggregate,
    UTXOAggregateId,
};
use bitview_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnId, Database, ReadableCloneableVec, ReadableColumnarVec, Rw,
    StorageMode,
};

use bitview_compute::{CACHE_BUDGET, ColumnarPerBlock, FiatType, LazyFiatPerBlock};

#[derive(Deref, DerefMut, Traversable)]
pub struct AdditiveAggregateFiatPerBlock<C: FiatType, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub values: ColumnarPerBlock<C, TermId, UTXOAggregate<LazyFiatPerBlock<C>>, M>,
}

impl<C: FiatType> AdditiveAggregateFiatPerBlock<C> {
    pub fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> Result<Self> {
        let values = ColumnarPerBlock::forced_import(
            db,
            &format!("{metric}_cents_by_term"),
            version,
            |source| {
                let source = source.clone();
                UTXOAggregate::from_fn(|aggregate| {
                    let name = CohortContext::Utxo.metric_name(
                        aggregate.select(&UTXO_AGGREGATE_FILTERS),
                        aggregate.select(&UTXO_AGGREGATE_NAMES).id,
                        metric,
                    );
                    let cents = match aggregate {
                        UTXOAggregateId::All => CACHE_BUDGET
                            .wrap(source.sum_columns(
                                &format!("{name}_cents"),
                                version,
                                TermId::ALL.iter().copied(),
                            ))
                            .read_only_boxed_clone(),
                        UTXOAggregateId::Sth => source
                            .column(&format!("{name}_cents"), version, TermId::Short)
                            .read_only_boxed_clone(),
                        UTXOAggregateId::Lth => source
                            .column(&format!("{name}_cents"), version, TermId::Long)
                            .read_only_boxed_clone(),
                    };
                    LazyFiatPerBlock::from_boxed_cents_source(&name, version, cents, mappings)
                })
            },
        )?;
        Ok(Self { values })
    }

    #[inline(always)]
    pub fn push(&mut self, row: UTXOAggregate<C>) {
        self.values.push(ByTerm {
            short: row.sth,
            long: row.lth,
        });
    }

    pub fn len(&self) -> usize {
        self.values.height.len()
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.values.stored_mut()
    }
}

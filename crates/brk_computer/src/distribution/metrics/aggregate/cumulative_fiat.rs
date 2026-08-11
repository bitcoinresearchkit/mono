use brk_cohort::{
    ByTerm, TermId, UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES, UTXOAggregate, UTXOAggregateId,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnId, Database, ReadableCloneableVec, ReadableColumnarVec, Rw,
    StorageMode,
};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarPerBlockCumulativeRolling, FiatType,
        LazyFiatPerBlockCumulativeWithSums, Windows,
    },
};

use super::utxo_metric_name;

#[derive(Deref, DerefMut, Traversable)]
pub struct AdditiveAggregateFiatPerBlockCumulativeWithSums<C: FiatType, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub values: ColumnarPerBlockCumulativeRolling<
        C,
        TermId,
        UTXOAggregate<LazyFiatPerBlockCumulativeWithSums<C>>,
        M,
    >,
}

impl<C: FiatType> AdditiveAggregateFiatPerBlockCumulativeWithSums<C> {
    pub(crate) fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let values = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            &format!("{metric}_cumulative_cents_by_term"),
            version,
            |source| {
                let source = source.clone();
                UTXOAggregate::from_fn(|id| {
                    let name = utxo_metric_name(
                        id.select(&UTXO_AGGREGATE_FILTERS),
                        id.select(&UTXO_AGGREGATE_NAMES).id,
                        metric,
                    );
                    let cumulative = match id {
                        UTXOAggregateId::All => source
                            .sum_columns(
                                &format!("{name}_cumulative_cents"),
                                version,
                                TermId::ALL.iter().copied(),
                            )
                            .read_only_boxed_clone(),
                        UTXOAggregateId::Sth => source
                            .column(&format!("{name}_cumulative_cents"), version, TermId::Short)
                            .read_only_boxed_clone(),
                        UTXOAggregateId::Lth => source
                            .column(&format!("{name}_cumulative_cents"), version, TermId::Long)
                            .read_only_boxed_clone(),
                    };
                    LazyFiatPerBlockCumulativeWithSums::from_boxed_cumulative_cents_source(
                        &name,
                        version,
                        cumulative,
                        indexes,
                        cached_starts,
                    )
                })
            },
        )?;
        Ok(Self { values })
    }

    #[inline(always)]
    pub(crate) fn push_block(&mut self, row: UTXOAggregate<C>) {
        self.values.push_block(ByTerm {
            short: row.sth,
            long: row.lth,
        });
    }

    pub(crate) fn len(&self) -> usize {
        self.values.cumulative.len()
    }

    pub(crate) fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.values.stored_mut()
    }
}

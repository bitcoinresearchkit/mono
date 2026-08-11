use std::ops::AddAssign;

use brk_cohort::{
    Filter, OVER_AGE_FILTERS, OVER_AMOUNT_FILTERS, OverAgeId, OverAmountId, UNDER_AGE_FILTERS,
    UNDER_AMOUNT_FILTERS, UTXO_AGGREGATE_FILTERS, UTXOAggregateId, UnderAgeId, UnderAmountId,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnId, ColumnarVec, Database, EagerVec, ImportableVec, PcoVec,
    PcoVecValue, ReadOnlyClone, ReadableBoxedVec, ReadableCloneableVec, ReadableColumnarVec, Rw,
    StorageMode, WritableVec,
};

use super::{UTXOAggregateRows, UTXOColumnarMetric, UTXORows};

#[derive(Traversable)]
pub struct ExactUTXOColumnarMetric<T, M: StorageMode = Rw>
where
    T: PcoVecValue,
{
    #[traversable(flatten)]
    pub direct: UTXOColumnarMetric<T, M>,
    pub aggregate_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, UTXOAggregateId>>>,
    pub under_age_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, UnderAgeId>>>,
    pub over_age_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, OverAgeId>>>,
    pub under_amount_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, UnderAmountId>>>,
    pub over_amount_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, OverAmountId>>>,
}

impl<T> ExactUTXOColumnarMetric<T>
where
    T: PcoVecValue + AddAssign,
{
    pub(crate) fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        let direct = UTXOColumnarMetric::forced_import(db, name, version)?;
        let version = version + Version::ONE;

        Ok(Self {
            direct,
            aggregate_matrix: EagerVec::forced_import(
                db,
                &format!("{name}_by_aggregate"),
                version,
            )?,
            under_age_matrix: EagerVec::forced_import(
                db,
                &format!("utxos_{name}_by_under_age"),
                version,
            )?,
            over_age_matrix: EagerVec::forced_import(
                db,
                &format!("utxos_{name}_by_over_age"),
                version,
            )?,
            under_amount_matrix: EagerVec::forced_import(
                db,
                &format!("utxos_{name}_by_under_amount"),
                version,
            )?,
            over_amount_matrix: EagerVec::forced_import(
                db,
                &format!("utxos_{name}_by_over_amount"),
                version,
            )?,
        })
    }

    pub(crate) fn source(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        if let Some(source) = self.direct.direct_source(filter, name, version) {
            return Some(source);
        }

        match filter {
            Filter::All | Filter::Term(_) => UTXOAggregateId::ALL
                .iter()
                .copied()
                .find(|id| id.select(&UTXO_AGGREGATE_FILTERS) == filter)
                .map(|id| Self::column(&self.aggregate_matrix, name, version, id)),
            Filter::Time(_) => UnderAgeId::ALL
                .iter()
                .copied()
                .find(|id| id.select(&UNDER_AGE_FILTERS) == filter)
                .map(|id| Self::column(&self.under_age_matrix, name, version, id))
                .or_else(|| {
                    OverAgeId::ALL
                        .iter()
                        .copied()
                        .find(|id| id.select(&OVER_AGE_FILTERS) == filter)
                        .map(|id| Self::column(&self.over_age_matrix, name, version, id))
                }),
            Filter::Amount(_) => UnderAmountId::ALL
                .iter()
                .copied()
                .find(|id| id.select(&UNDER_AMOUNT_FILTERS) == filter)
                .map(|id| Self::column(&self.under_amount_matrix, name, version, id))
                .or_else(|| {
                    OverAmountId::ALL
                        .iter()
                        .copied()
                        .find(|id| id.select(&OVER_AMOUNT_FILTERS) == filter)
                        .map(|id| Self::column(&self.over_amount_matrix, name, version, id))
                }),
            Filter::Epoch(_) | Filter::Class(_) | Filter::Entry(_) | Filter::Type(_) => None,
        }
    }

    fn column<C>(
        matrix: &EagerVec<ColumnarVec<PcoVec<Height, T>, C>>,
        name: &str,
        version: Version,
        column: C,
    ) -> ReadableBoxedVec<Height, T>
    where
        C: ColumnId,
    {
        matrix
            .read_only_clone()
            .column(name, version, column)
            .read_only_boxed_clone()
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, direct: UTXORows<T>, aggregates: UTXOAggregateRows<T>) {
        self.direct.push(direct);
        self.aggregate_matrix.push(aggregates.aggregate);
        self.under_age_matrix.push(aggregates.under_age);
        self.over_age_matrix.push(aggregates.over_age);
        self.under_amount_matrix.push(aggregates.under_amount);
        self.over_amount_matrix.push(aggregates.over_amount);
    }

    pub(crate) fn min_len(&self) -> usize {
        self.direct
            .min_len()
            .min(self.aggregate_matrix.len())
            .min(self.under_age_matrix.len())
            .min(self.over_age_matrix.len())
            .min(self.under_amount_matrix.len())
            .min(self.over_amount_matrix.len())
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let Self {
            direct,
            aggregate_matrix,
            under_age_matrix,
            over_age_matrix,
            under_amount_matrix,
            over_amount_matrix,
        } = self;
        let mut vecs = direct.collect_vecs_mut();
        vecs.extend([
            aggregate_matrix as &mut dyn AnyStoredVec,
            under_age_matrix,
            over_age_matrix,
            under_amount_matrix,
            over_amount_matrix,
        ]);
        vecs
    }
}

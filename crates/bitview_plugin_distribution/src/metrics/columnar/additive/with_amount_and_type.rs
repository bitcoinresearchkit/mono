use brk_error::Result;

use std::ops::AddAssign;

use bitview_cohort::{
    AgeRangeId, AmountRangeId, ClassId, EntryId, EpochId, Filter, OVER_AMOUNT_FILTERS,
    SpendableTypeId, UNDER_AMOUNT_FILTERS,
};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnarVec, Database, EagerVec, ImportableVec, PcoVec, PcoVecValue,
    ReadOnlyClone, ReadableBoxedVec, ReadableVec, Rw, StorageMode, WritableVec,
};

use super::super::UTXORows;
use super::UTXOColumnarMetricWithoutAmountOrType;

#[derive(Traversable)]
pub struct UTXOColumnarMetric<T, M: StorageMode = Rw>
where
    T: PcoVecValue,
{
    /// Height-indexed matrix with one column per exact UTXO age range, ordered
    /// from youngest to oldest.
    pub age_range_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, AgeRangeId>>>,
    /// Height-indexed matrix with one column per subsidy-halving epoch, ordered
    /// from earliest to latest.
    pub epoch_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, EpochId>>>,
    /// Height-indexed matrix with one column per output-creation year, in
    /// chronological order.
    pub class_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, ClassId>>>,
    /// Height-indexed matrix with discount-entry followed by premium-entry
    /// UTXOs.
    pub entry_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, EntryId>>>,
    /// Height-indexed matrix with one column per spendable BRK output type, in
    /// canonical `SpendableTypeId` order.
    pub type_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, SpendableTypeId>>>,
    /// Height-indexed matrix with one column per exact UTXO value range,
    /// ordered from smallest to largest.
    pub amount_range_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, AmountRangeId>>>,
}

impl<T> UTXOColumnarMetric<T>
where
    T: PcoVecValue + AddAssign,
{
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        let version = version + Version::ONE;
        Ok(Self {
            age_range_matrix: EagerVec::forced_import(
                db,
                &format!("utxos_{name}_by_age_range"),
                version,
            )?,
            epoch_matrix: EagerVec::forced_import(db, &format!("{name}_by_epoch"), version)?,
            class_matrix: EagerVec::forced_import(db, &format!("{name}_by_class"), version)?,
            entry_matrix: EagerVec::forced_import(db, &format!("{name}_by_entry"), version)?,
            type_matrix: EagerVec::forced_import(db, &format!("{name}_by_type"), version)?,
            amount_range_matrix: EagerVec::forced_import(
                db,
                &format!("utxos_{name}_by_amount_range"),
                version,
            )?,
        })
    }

    pub fn additive_source(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        self.direct_source(filter, name, version)
            .or_else(|| self.aggregate_amount_source(filter, name, version))
            .or_else(|| {
                UTXOColumnarMetricWithoutAmountOrType::aggregate_source_from(
                    &self.age_range_matrix.read_only_clone(),
                    filter,
                    name,
                    version,
                )
            })
    }

    pub fn direct_source(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        match filter {
            Filter::Amount(_) => AmountRangeId::matching(filter).map(|id| {
                UTXOColumnarMetricWithoutAmountOrType::column(
                    &self.amount_range_matrix.read_only_clone(),
                    name,
                    version,
                    id,
                )
            }),
            Filter::Type(output_type) => {
                SpendableTypeId::from_output_type(*output_type).map(|id| {
                    UTXOColumnarMetricWithoutAmountOrType::column(
                        &self.type_matrix.read_only_clone(),
                        name,
                        version,
                        id,
                    )
                })
            }
            _ => UTXOColumnarMetricWithoutAmountOrType::direct_source_from(
                &self.age_range_matrix.read_only_clone(),
                &self.epoch_matrix.read_only_clone(),
                &self.class_matrix.read_only_clone(),
                &self.entry_matrix.read_only_clone(),
                filter,
                name,
                version,
            ),
        }
    }

    fn aggregate_amount_source(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        let matrix = self.amount_range_matrix.read_only_clone();
        match filter {
            Filter::Amount(_) => UNDER_AMOUNT_FILTERS
                .iter()
                .chain(OVER_AMOUNT_FILTERS.iter())
                .find(|candidate| *candidate == filter)
                .map(|filter| {
                    UTXOColumnarMetricWithoutAmountOrType::sum(
                        &matrix,
                        name,
                        version,
                        AmountRangeId::included_by(filter),
                    )
                }),
            _ => None,
        }
    }

    pub fn min_len(&self) -> usize {
        self.age_range_matrix
            .len()
            .min(self.epoch_matrix.len())
            .min(self.class_matrix.len())
            .min(self.entry_matrix.len())
            .min(self.type_matrix.len())
            .min(self.amount_range_matrix.len())
    }

    #[inline(always)]
    pub fn push(&mut self, rows: UTXORows<T>) {
        let UTXORows {
            age_range,
            epoch,
            class,
            entry,
            amount_range,
            type_,
        } = rows;
        self.age_range_matrix.push(age_range);
        self.epoch_matrix.push(epoch);
        self.class_matrix.push(class);
        self.entry_matrix.push(entry);
        self.type_matrix.push(type_);
        self.amount_range_matrix.push(amount_range);
    }

    pub fn collect_last(&self) -> Option<UTXORows<T>>
    where
        T: Default,
    {
        Some(UTXORows {
            age_range: self.age_range_matrix.collect_last()?,
            epoch: self.epoch_matrix.collect_last()?,
            class: self.class_matrix.collect_last()?,
            entry: self.entry_matrix.collect_last()?,
            amount_range: self.amount_range_matrix.collect_last()?,
            type_: self.type_matrix.collect_last()?,
        })
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        vec![
            &mut self.age_range_matrix,
            &mut self.epoch_matrix,
            &mut self.class_matrix,
            &mut self.entry_matrix,
            &mut self.type_matrix,
            &mut self.amount_range_matrix,
        ]
    }
}

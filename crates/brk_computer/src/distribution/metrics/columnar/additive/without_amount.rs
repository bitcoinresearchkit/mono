use std::ops::AddAssign;

use brk_cohort::{
    AgeRangeId, ClassId, EntryId, EpochId, Filter, SPENDABLE_TYPE_FILTERS, SpendableTypeId,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnarVec, Database, EagerVec, ImportableVec, PcoVec, PcoVecValue,
    ReadOnlyClone, ReadableBoxedVec, Rw, StorageMode, WritableVec,
};

use super::super::UTXORows;
use super::UTXOColumnarMetricWithoutAmountOrType;

#[derive(Traversable)]
pub struct UTXOColumnarMetricWithoutAmount<T, M: StorageMode = Rw>
where
    T: PcoVecValue,
{
    pub age_range_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, AgeRangeId>>>,
    pub epoch_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, EpochId>>>,
    pub class_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, ClassId>>>,
    pub entry_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, EntryId>>>,
    pub type_matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, SpendableTypeId>>>,
}

impl<T> UTXOColumnarMetricWithoutAmount<T>
where
    T: PcoVecValue + AddAssign,
{
    pub(crate) fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
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
        })
    }

    pub(crate) fn additive_source(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        self.direct_source(filter, name, version).or_else(|| {
            UTXOColumnarMetricWithoutAmountOrType::aggregate_source_from(
                &self.age_range_matrix.read_only_clone(),
                filter,
                name,
                version,
            )
        })
    }

    pub(super) fn direct_source(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        match filter {
            Filter::Type(output_type) => {
                SpendableTypeId::from_output_type(*output_type).map(|id| {
                    debug_assert_eq!(id.select(&SPENDABLE_TYPE_FILTERS), filter);
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

    pub(crate) fn min_len(&self) -> usize {
        self.age_range_matrix
            .len()
            .min(self.epoch_matrix.len())
            .min(self.class_matrix.len())
            .min(self.entry_matrix.len())
            .min(self.type_matrix.len())
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, rows: UTXORows<T>) {
        let UTXORows {
            age_range,
            epoch,
            class,
            entry,
            type_,
            ..
        } = rows;
        self.age_range_matrix.push(age_range);
        self.epoch_matrix.push(epoch);
        self.class_matrix.push(class);
        self.entry_matrix.push(entry);
        self.type_matrix.push(type_);
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        vec![
            &mut self.age_range_matrix,
            &mut self.epoch_matrix,
            &mut self.class_matrix,
            &mut self.entry_matrix,
            &mut self.type_matrix,
        ]
    }
}

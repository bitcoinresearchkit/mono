use brk_error::Result;

use std::ops::AddAssign;

use bitview_cohort::{
    AgeRange, AgeRangeId, AmountRange, ByEntry, ByEpoch, CLASS_FILTERS, Class, ClassId,
    ENTRY_FILTERS, EPOCH_FILTERS, EntryId, EpochId, Filter, OVER_AGE_FILTERS, SpendableType,
    TERM_FILTERS, Term, UNDER_AGE_FILTERS,
};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnId, ColumnarVec, Database, EagerVec, ImportableVec, PcoVec,
    PcoVecValue, ReadOnlyClone, ReadOnlyColumnarVec, ReadableBoxedVec, ReadableCloneableVec,
    ReadableColumnarVec, ReadableVec, Rw, StorageMode, WritableVec,
};

use super::super::UTXORows;
use bitview_compute::CACHE_BUDGET;

#[derive(Traversable)]
pub struct UTXOColumnarMetricWithoutAmountOrType<T, M: StorageMode = Rw>
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
}

impl<T> UTXOColumnarMetricWithoutAmountOrType<T>
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
        })
    }

    pub fn additive_source(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        self.direct_source(filter, name, version)
            .or_else(|| self.aggregate_source(filter, name, version))
    }

    pub fn direct_source(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        Self::direct_source_from(
            &self.age_range_matrix.read_only_clone(),
            &self.epoch_matrix.read_only_clone(),
            &self.class_matrix.read_only_clone(),
            &self.entry_matrix.read_only_clone(),
            filter,
            name,
            version,
        )
    }

    pub fn direct_source_from(
        age_range_matrix: &ReadOnlyColumnarVec<PcoVec<Height, T>, AgeRangeId>,
        epoch_matrix: &ReadOnlyColumnarVec<PcoVec<Height, T>, EpochId>,
        class_matrix: &ReadOnlyColumnarVec<PcoVec<Height, T>, ClassId>,
        entry_matrix: &ReadOnlyColumnarVec<PcoVec<Height, T>, EntryId>,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        match filter {
            Filter::Time(_) => AgeRangeId::matching(filter)
                .map(|id| Self::column(age_range_matrix, name, version, id)),
            Filter::Epoch(_) => EpochId::ALL
                .iter()
                .copied()
                .find(|id| id.select(&EPOCH_FILTERS) == filter)
                .map(|id| Self::column(epoch_matrix, name, version, id)),
            Filter::Class(_) => ClassId::ALL
                .iter()
                .copied()
                .find(|id| id.select(&CLASS_FILTERS) == filter)
                .map(|id| Self::column(class_matrix, name, version, id)),
            Filter::Entry(_) => EntryId::ALL
                .iter()
                .copied()
                .find(|id| id.select(&ENTRY_FILTERS) == filter)
                .map(|id| Self::column(entry_matrix, name, version, id)),
            Filter::All | Filter::Term(_) | Filter::Amount(_) | Filter::Type(_) => None,
        }
    }

    pub fn aggregate_source(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        Self::aggregate_source_from(
            &self.age_range_matrix.read_only_clone(),
            filter,
            name,
            version,
        )
    }

    pub fn aggregate_source_from(
        age_range_matrix: &ReadOnlyColumnarVec<PcoVec<Height, T>, AgeRangeId>,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<ReadableBoxedVec<Height, T>> {
        match filter {
            Filter::All => Some(Self::sum(
                age_range_matrix,
                name,
                version,
                AgeRangeId::ALL.iter().copied(),
            )),
            Filter::Term(term) => {
                let term = match term {
                    Term::Sth => &TERM_FILTERS.short,
                    Term::Lth => &TERM_FILTERS.long,
                };
                Some(Self::sum(
                    age_range_matrix,
                    name,
                    version,
                    AgeRangeId::ALL
                        .iter()
                        .copied()
                        .filter(|column| term.includes(column.filter())),
                ))
            }
            Filter::Time(_) => UNDER_AGE_FILTERS
                .iter()
                .chain(OVER_AGE_FILTERS.iter())
                .find(|candidate| *candidate == filter)
                .map(|filter| {
                    Self::sum(
                        age_range_matrix,
                        name,
                        version,
                        AgeRangeId::included_by(filter),
                    )
                }),
            Filter::Amount(_)
            | Filter::Epoch(_)
            | Filter::Class(_)
            | Filter::Entry(_)
            | Filter::Type(_) => None,
        }
    }

    pub fn column<C>(
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, C>,
        name: &str,
        version: Version,
        column: C,
    ) -> ReadableBoxedVec<Height, T>
    where
        C: ColumnId,
    {
        source.column(name, version, column).read_only_boxed_clone()
    }

    pub fn sum<C>(
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, C>,
        name: &str,
        version: Version,
        columns: impl IntoIterator<Item = C>,
    ) -> ReadableBoxedVec<Height, T>
    where
        C: ColumnId,
    {
        CACHE_BUDGET
            .wrap(source.sum_columns(name, version, columns))
            .read_only_boxed_clone()
    }

    pub fn min_len(&self) -> usize {
        self.age_range_matrix
            .len()
            .min(self.epoch_matrix.len())
            .min(self.class_matrix.len())
            .min(self.entry_matrix.len())
    }

    pub fn push_parts(
        &mut self,
        age_range: AgeRange<T>,
        epoch: ByEpoch<T>,
        class: Class<T>,
        entry: ByEntry<T>,
    ) {
        self.age_range_matrix.push(age_range);
        self.epoch_matrix.push(epoch);
        self.class_matrix.push(class);
        self.entry_matrix.push(entry);
    }

    #[inline(always)]
    pub fn push(&mut self, rows: UTXORows<T>) {
        self.push_parts(rows.age_range, rows.epoch, rows.class, rows.entry);
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
            amount_range: AmountRange::default(),
            type_: SpendableType::default(),
        })
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        vec![
            &mut self.age_range_matrix,
            &mut self.epoch_matrix,
            &mut self.class_matrix,
            &mut self.entry_matrix,
        ]
    }
}

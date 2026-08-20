use brk_error::Result;

use bitview_cohort::{
    AgeRangeId, CLASS_FILTERS, ClassId, ENTRY_FILTERS, EPOCH_FILTERS, EntryId, EpochId, Filter,
    OVER_AGE_FILTERS, TERM_FILTERS, Term, UNDER_AGE_FILTERS,
};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, ColumnId, Database, ReadableBoxedVec, Rw, StorageMode};

use bitview_compute::ColumnarValuePerBlockCumulativeRolling;

use super::super::UTXORows;

#[derive(Traversable)]
pub struct CumulativeUTXOValueColumnarMetricWithoutAmountOrType<M: StorageMode = Rw> {
    pub age_range: ColumnarValuePerBlockCumulativeRolling<AgeRangeId, (), M>,
    pub epoch: ColumnarValuePerBlockCumulativeRolling<EpochId, (), M>,
    pub class: ColumnarValuePerBlockCumulativeRolling<ClassId, (), M>,
    pub entry: ColumnarValuePerBlockCumulativeRolling<EntryId, (), M>,
}

impl CumulativeUTXOValueColumnarMetricWithoutAmountOrType {
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        let version = version + Version::ONE;
        Ok(Self {
            age_range: Self::import(db, &format!("utxos_{name}_by_age_range"), version)?,
            epoch: Self::import(db, &format!("{name}_by_epoch"), version)?,
            class: Self::import(db, &format!("{name}_by_class"), version)?,
            entry: Self::import(db, &format!("{name}_by_entry"), version)?,
        })
    }

    fn import<C>(
        db: &Database,
        name: &str,
        version: Version,
    ) -> Result<ColumnarValuePerBlockCumulativeRolling<C, ()>>
    where
        C: ColumnId,
    {
        ColumnarValuePerBlockCumulativeRolling::forced_import(db, name, version, |_, _| ())
    }

    pub fn sources(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<(
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    )> {
        self.direct_sources(filter, name, version)
            .or_else(|| self.aggregate_sources(filter, name, version))
    }

    pub fn direct_sources(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<(
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    )> {
        Self::direct_sources_from(
            &self.age_range,
            &self.epoch,
            &self.class,
            &self.entry,
            filter,
            name,
            version,
        )
    }

    pub fn direct_sources_from(
        age_range: &ColumnarValuePerBlockCumulativeRolling<AgeRangeId, ()>,
        epoch: &ColumnarValuePerBlockCumulativeRolling<EpochId, ()>,
        class: &ColumnarValuePerBlockCumulativeRolling<ClassId, ()>,
        entry: &ColumnarValuePerBlockCumulativeRolling<EntryId, ()>,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<(
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    )> {
        match filter {
            Filter::Time(_) => AgeRangeId::matching(filter)
                .map(|column| Self::matrix_sources(age_range, name, version, [column])),
            Filter::Epoch(_) => EpochId::ALL
                .iter()
                .copied()
                .find(|column| column.select(&EPOCH_FILTERS) == filter)
                .map(|column| Self::matrix_sources(epoch, name, version, [column])),
            Filter::Class(_) => ClassId::ALL
                .iter()
                .copied()
                .find(|column| column.select(&CLASS_FILTERS) == filter)
                .map(|column| Self::matrix_sources(class, name, version, [column])),
            Filter::Entry(_) => EntryId::ALL
                .iter()
                .copied()
                .find(|column| column.select(&ENTRY_FILTERS) == filter)
                .map(|column| Self::matrix_sources(entry, name, version, [column])),
            Filter::All | Filter::Term(_) | Filter::Amount(_) | Filter::Type(_) => None,
        }
    }

    pub fn aggregate_sources(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<(
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    )> {
        let columns = Self::age_columns(filter)?;
        Some(Self::matrix_sources(
            &self.age_range,
            name,
            version,
            columns,
        ))
    }

    pub fn age_columns(filter: &Filter) -> Option<Vec<AgeRangeId>> {
        Some(match filter {
            Filter::All => AgeRangeId::ALL.to_vec(),
            Filter::Term(term) => {
                let filter = match term {
                    Term::Sth => &TERM_FILTERS.short,
                    Term::Lth => &TERM_FILTERS.long,
                };
                AgeRangeId::included_by(filter).collect()
            }
            Filter::Time(_) => UNDER_AGE_FILTERS
                .iter()
                .chain(OVER_AGE_FILTERS.iter())
                .find(|candidate| *candidate == filter)
                .map(|filter| AgeRangeId::included_by(filter).collect())?,
            Filter::Amount(_)
            | Filter::Epoch(_)
            | Filter::Class(_)
            | Filter::Entry(_)
            | Filter::Type(_) => return None,
        })
    }

    pub fn matrix_sources<C>(
        matrix: &ColumnarValuePerBlockCumulativeRolling<C, ()>,
        name: &str,
        version: Version,
        columns: impl IntoIterator<Item = C>,
    ) -> (
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    )
    where
        C: ColumnId,
    {
        matrix.sources(&format!("{name}_cumulative"), version, columns)
    }

    #[inline(always)]
    pub fn push_block(&mut self, sats: UTXORows<Sats>, cents: UTXORows<Cents>) {
        self.age_range.push_block(sats.age_range, cents.age_range);
        self.epoch.push_block(sats.epoch, cents.epoch);
        self.class.push_block(sats.class, cents.class);
        self.entry.push_block(sats.entry, cents.entry);
    }

    pub fn min_len(&self) -> usize {
        self.age_range
            .len()
            .min(self.epoch.len())
            .min(self.class.len())
            .min(self.entry.len())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let Self {
            age_range,
            epoch,
            class,
            entry,
        } = self;
        let mut vecs = age_range.collect_vecs_mut();
        vecs.extend(epoch.collect_vecs_mut());
        vecs.extend(class.collect_vecs_mut());
        vecs.extend(entry.collect_vecs_mut());
        vecs
    }
}

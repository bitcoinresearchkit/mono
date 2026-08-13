use brk_cohort::{
    AGE_RANGE_FILTERS, AgeRange, AgeRangeId, ByEntry, ByEpoch, CLASS_FILTERS, Class, ClassId,
    CohortContext, ENTRY_FILTERS, EPOCH_FILTERS, EntryId, EpochId, Filter, OVER_AGE_FILTERS,
    OverAge, OverAgeId, Term, UNDER_AGE_FILTERS, UTXOAggregate, UTXOAggregateId,
    UTXOGroupsWithoutAmountOrType, UnderAge, UnderAgeId,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, StoredF32, Version};
use vecdb::{
    AnyStoredVec, BinaryTransform, ColumnId, Database, Exit, PcoVec, ReadOnlyColumnarVec,
    ReadableCloneableVec, Rw, StorageMode,
};

use crate::{
    indexes,
    internal::{ColumnarPerBlock, Identity, LazyColumnPerBlock, LazyPerBlock, SoprRatio},
};

use super::Sopr24hInput;

const VERSION: Version = Version::ONE;

#[derive(Traversable)]
pub struct Sopr24hVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<LazyPerBlock<StoredF32>>,
    pub aggregate_matrix: ColumnarPerBlock<
        StoredF32,
        UTXOAggregateId,
        UTXOAggregate<LazyColumnPerBlock<StoredF32, UTXOAggregateId>>,
        M,
    >,
    pub age_range_matrix: ColumnarPerBlock<
        StoredF32,
        AgeRangeId,
        AgeRange<LazyColumnPerBlock<StoredF32, AgeRangeId>>,
        M,
    >,
    pub under_age_matrix: ColumnarPerBlock<
        StoredF32,
        UnderAgeId,
        UnderAge<LazyColumnPerBlock<StoredF32, UnderAgeId>>,
        M,
    >,
    pub over_age_matrix: ColumnarPerBlock<
        StoredF32,
        OverAgeId,
        OverAge<LazyColumnPerBlock<StoredF32, OverAgeId>>,
        M,
    >,
    pub epoch_matrix:
        ColumnarPerBlock<StoredF32, EpochId, ByEpoch<LazyColumnPerBlock<StoredF32, EpochId>>, M>,
    pub class_matrix:
        ColumnarPerBlock<StoredF32, ClassId, Class<LazyColumnPerBlock<StoredF32, ClassId>>, M>,
    pub entry_matrix:
        ColumnarPerBlock<StoredF32, EntryId, ByEntry<LazyColumnPerBlock<StoredF32, EntryId>>, M>,
}

impl Sopr24hVecs {
    pub fn forced_import(db: &Database, version: Version, indexes: &indexes::Vecs) -> Result<Self> {
        let version = version + VERSION;
        let matrix_version = version + Version::ONE;
        let aggregate_matrix = Self::import_matrix(
            db,
            "sopr_24h_by_aggregate",
            matrix_version,
            |name, source| UTXOAggregate {
                all: Self::column(name, matrix_version, source, UTXOAggregateId::All, indexes),
                sth: Self::column(name, matrix_version, source, UTXOAggregateId::Sth, indexes),
                lth: Self::column(name, matrix_version, source, UTXOAggregateId::Lth, indexes),
            },
        )?;
        let age_range_matrix = Self::import_matrix(
            db,
            "utxos_sopr_24h_by_age_range",
            matrix_version,
            |name, source| {
                AgeRange::from_fn(|column| {
                    Self::column(name, matrix_version, source, column, indexes)
                })
            },
        )?;
        let under_age_matrix = Self::import_matrix(
            db,
            "utxos_sopr_24h_by_under_age",
            matrix_version,
            |name, source| {
                UnderAge::from_fn(|column| {
                    Self::column(name, matrix_version, source, column, indexes)
                })
            },
        )?;
        let over_age_matrix = Self::import_matrix(
            db,
            "utxos_sopr_24h_by_over_age",
            matrix_version,
            |name, source| {
                OverAge::from_fn(|column| {
                    Self::column(name, matrix_version, source, column, indexes)
                })
            },
        )?;
        let epoch_matrix =
            Self::import_matrix(db, "sopr_24h_by_epoch", matrix_version, |name, source| {
                ByEpoch::from_fn(|column| {
                    Self::column(name, matrix_version, source, column, indexes)
                })
            })?;
        let class_matrix =
            Self::import_matrix(db, "sopr_24h_by_class", matrix_version, |name, source| {
                Class::from_fn(|column| Self::column(name, matrix_version, source, column, indexes))
            })?;
        let entry_matrix =
            Self::import_matrix(db, "sopr_24h_by_entry", matrix_version, |name, source| {
                ByEntry::from_fn(|column| {
                    Self::column(name, matrix_version, source, column, indexes)
                })
            })?;

        let cohorts = UTXOGroupsWithoutAmountOrType::new(|filter, cohort_name| {
            let name = CohortContext::Utxo.metric_name(&filter, cohort_name, "sopr_24h");
            let version = Self::cohort_version(version, &filter);
            match &filter {
                Filter::All => {
                    Self::logical_source(&aggregate_matrix.series.all, &name, version, indexes)
                }
                Filter::Term(Term::Sth) => {
                    Self::logical_source(&aggregate_matrix.series.sth, &name, version, indexes)
                }
                Filter::Term(Term::Lth) => {
                    Self::logical_source(&aggregate_matrix.series.lth, &name, version, indexes)
                }
                Filter::Time(_) => AgeRangeId::ALL
                    .iter()
                    .copied()
                    .find(|id| id.select(&AGE_RANGE_FILTERS) == &filter)
                    .map(|id| {
                        Self::logical_source(
                            id.select(&age_range_matrix.series),
                            &name,
                            version,
                            indexes,
                        )
                    })
                    .or_else(|| {
                        UnderAgeId::ALL
                            .iter()
                            .copied()
                            .find(|id| id.select(&UNDER_AGE_FILTERS) == &filter)
                            .map(|id| {
                                Self::logical_source(
                                    id.select(&under_age_matrix.series),
                                    &name,
                                    version,
                                    indexes,
                                )
                            })
                    })
                    .or_else(|| {
                        OverAgeId::ALL
                            .iter()
                            .copied()
                            .find(|id| id.select(&OVER_AGE_FILTERS) == &filter)
                            .map(|id| {
                                Self::logical_source(
                                    id.select(&over_age_matrix.series),
                                    &name,
                                    version,
                                    indexes,
                                )
                            })
                    })
                    .expect("supported SOPR time cohort"),
                Filter::Epoch(_) => EpochId::ALL
                    .iter()
                    .copied()
                    .find(|id| id.select(&EPOCH_FILTERS) == &filter)
                    .map(|id| {
                        Self::logical_source(
                            id.select(&epoch_matrix.series),
                            &name,
                            version,
                            indexes,
                        )
                    })
                    .expect("supported SOPR epoch cohort"),
                Filter::Class(_) => ClassId::ALL
                    .iter()
                    .copied()
                    .find(|id| id.select(&CLASS_FILTERS) == &filter)
                    .map(|id| {
                        Self::logical_source(
                            id.select(&class_matrix.series),
                            &name,
                            version,
                            indexes,
                        )
                    })
                    .expect("supported SOPR class cohort"),
                Filter::Entry(_) => EntryId::ALL
                    .iter()
                    .copied()
                    .find(|id| id.select(&ENTRY_FILTERS) == &filter)
                    .map(|id| {
                        Self::logical_source(
                            id.select(&entry_matrix.series),
                            &name,
                            version,
                            indexes,
                        )
                    })
                    .expect("supported SOPR entry cohort"),
                Filter::Amount(_) | Filter::Type(_) => unreachable!("unsupported SOPR cohort"),
            }
        });

        Ok(Self {
            cohorts,
            aggregate_matrix,
            age_range_matrix,
            under_age_matrix,
            over_age_matrix,
            epoch_matrix,
            class_matrix,
            entry_matrix,
        })
    }

    fn import_matrix<C: ColumnId, S: Clone>(
        db: &Database,
        name: &str,
        version: Version,
        build_series: impl FnOnce(&str, &ReadOnlyColumnarVec<PcoVec<Height, StoredF32>, C>) -> S,
    ) -> Result<ColumnarPerBlock<StoredF32, C, S>> {
        ColumnarPerBlock::forced_import(db, name, version, |source| build_series(name, source))
    }

    fn column<C: ColumnId>(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, StoredF32>, C>,
        column: C,
        indexes: &indexes::Vecs,
    ) -> LazyColumnPerBlock<StoredF32, C> {
        LazyColumnPerBlock::new(
            &format!("{name}_column_{}", column.index()),
            version,
            source,
            column,
            indexes,
        )
    }

    fn logical_source<C: ColumnId>(
        source: &LazyColumnPerBlock<StoredF32, C>,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> LazyPerBlock<StoredF32> {
        LazyPerBlock::from_uncached_boxed_height_source::<Identity<StoredF32>>(
            name,
            version,
            source.height.read_only_boxed_clone(),
            indexes,
        )
    }

    fn cohort_version(base: Version, filter: &Filter) -> Version {
        base + Version::ONE
            + if matches!(filter, Filter::All) {
                Version::ONE
            } else {
                Version::ZERO
            }
    }

    pub fn compute(
        &mut self,
        max_from: Height,
        inputs: &UTXOGroupsWithoutAmountOrType<Sopr24hInput>,
        exit: &Exit,
    ) -> Result<()> {
        Self::compute_matrix(
            &mut self.aggregate_matrix,
            inputs,
            |column, inputs| match column {
                UTXOAggregateId::All => &inputs.all,
                UTXOAggregateId::Sth => &inputs.term.short,
                UTXOAggregateId::Lth => &inputs.term.long,
            },
            max_from,
            exit,
        )?;
        Self::compute_matrix(
            &mut self.age_range_matrix,
            &inputs.age.range,
            |column, inputs| column.select(inputs),
            max_from,
            exit,
        )?;
        Self::compute_matrix(
            &mut self.under_age_matrix,
            &inputs.age.under,
            |column, inputs| column.select(inputs),
            max_from,
            exit,
        )?;
        Self::compute_matrix(
            &mut self.over_age_matrix,
            &inputs.age.over,
            |column, inputs| column.select(inputs),
            max_from,
            exit,
        )?;
        Self::compute_matrix(
            &mut self.epoch_matrix,
            &inputs.epoch,
            |column, inputs| column.select(inputs),
            max_from,
            exit,
        )?;
        Self::compute_matrix(
            &mut self.class_matrix,
            &inputs.class,
            |column, inputs| column.select(inputs),
            max_from,
            exit,
        )?;
        Self::compute_matrix(
            &mut self.entry_matrix,
            &inputs.entry,
            |column, inputs| column.select(inputs),
            max_from,
            exit,
        )
    }

    fn compute_matrix<C: ColumnId, S: Clone, I>(
        target: &mut ColumnarPerBlock<StoredF32, C, S>,
        inputs: &I,
        select: impl for<'a> Fn(C, &'a I) -> &'a Sopr24hInput,
        max_from: Height,
        exit: &Exit,
    ) -> Result<()> {
        target.compute_columns2(
            max_from,
            |column| &select(column, inputs).transfer_volume,
            |column| &select(column, inputs).value_destroyed,
            |_, transfer_volume, value_destroyed| {
                SoprRatio::apply(transfer_volume, value_destroyed)
            },
            exit,
        )
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        vec![
            self.aggregate_matrix.stored_mut(),
            self.age_range_matrix.stored_mut(),
            self.under_age_matrix.stored_mut(),
            self.over_age_matrix.stored_mut(),
            self.epoch_matrix.stored_mut(),
            self.class_matrix.stored_mut(),
            self.entry_matrix.stored_mut(),
        ]
    }
}

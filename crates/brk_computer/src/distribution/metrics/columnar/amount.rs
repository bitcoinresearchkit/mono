use std::ops::AddAssign;

use brk_cohort::{Amount, AmountRange, AmountRangeId, CohortContext, Filter};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnarVec, Database, EagerVec, ImportableVec, PcoVec, PcoVecValue,
    ReadOnlyClone, ReadOnlyColumnarVec, ReadableBoxedVec, ReadableCloneableVec,
    ReadableColumnarVec, ReadableVec, Rw, StorageMode, WritableVec,
};

#[derive(Deref, DerefMut, Traversable)]
pub struct ColumnarAmount<T, S: Clone, M: StorageMode = Rw>
where
    T: PcoVecValue,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: Amount<S>,
    pub matrix: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, AmountRangeId>>>,
    #[traversable(skip)]
    last: Option<(usize, AmountRange<T>)>,
}

impl<T, S: Clone> ColumnarAmount<T, S>
where
    T: PcoVecValue + AddAssign,
{
    pub(crate) fn forced_import(
        db: &Database,
        matrix_name: &str,
        context: CohortContext,
        metric: &str,
        version: Version,
        mut build: impl FnMut(&str, ReadableBoxedVec<Height, T>) -> S,
    ) -> Result<Self> {
        let matrix = EagerVec::forced_import(db, matrix_name, version)?;
        let source: ReadOnlyColumnarVec<PcoVec<Height, T>, AmountRangeId> =
            matrix.read_only_clone();

        let series = Amount::new(|filter, cohort_name| {
            let name = Self::metric_name(context, &filter, cohort_name, metric);
            let source = match AmountRangeId::matching(&filter) {
                Some(column) => source
                    .column(&name, version, column)
                    .read_only_boxed_clone(),
                None => source
                    .sum_columns(&name, version, AmountRangeId::included_by(&filter))
                    .read_only_boxed_clone(),
            };
            build(&name, source)
        });

        Ok(Self {
            series,
            matrix,
            last: None,
        })
    }

    fn metric_name(
        context: CohortContext,
        filter: &Filter,
        cohort_name: &str,
        metric: &str,
    ) -> String {
        context.metric_name(filter, cohort_name, metric)
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, row: AmountRange<T>) {
        self.matrix.push(row);
    }

    #[inline(always)]
    pub(crate) fn push_cumulative(&mut self, delta: &AmountRange<T>)
    where
        T: AddAssign + Default,
    {
        let len = self.matrix.len();
        let mut cumulative = match self.last.take() {
            Some((cached_len, values)) if cached_len == len => values,
            _ => self.matrix.collect_last().unwrap_or_default(),
        };
        for (value, &delta) in cumulative.iter_mut().zip(delta.iter()) {
            *value += delta;
        }
        self.matrix.push(cumulative.clone());
        self.last = Some((len + 1, cumulative));
    }

    pub(crate) fn len(&self) -> usize {
        self.matrix.len()
    }

    pub(crate) fn reset(&mut self) -> Result<()> {
        self.last = None;
        self.matrix.reset().map_err(Into::into)
    }

    pub(crate) fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.last = None;
        &mut self.matrix
    }
}

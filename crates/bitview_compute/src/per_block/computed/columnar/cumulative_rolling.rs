use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnId, ColumnarVec, Database, EagerVec, Exit, ImportableVec, PcoVec,
    ReadOnlyClone, ReadOnlyColumnarVec, ReadableVec, Rw, StorageMode, VecValue, WritableVec,
};

use crate::NumericValue;

#[derive(Deref, DerefMut, Traversable)]
pub struct ColumnarPerBlockCumulativeRolling<T, C, S: Clone, M: StorageMode = Rw>
where
    T: NumericValue,
    C: ColumnId,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: S,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, C>>>,
    #[traversable(skip)]
    last_cumulative: Option<(usize, C::Row<T>)>,
}

impl<T, C, S: Clone> ColumnarPerBlockCumulativeRolling<T, C, S>
where
    T: NumericValue,
    C: ColumnId,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        build_series: impl FnOnce(&ReadOnlyColumnarVec<PcoVec<Height, T>, C>) -> S,
    ) -> Result<Self> {
        let cumulative = EagerVec::forced_import(db, name, version)?;
        let last_cumulative = cumulative.collect_last().map(|row| (cumulative.len(), row));
        let series = build_series(&cumulative.read_only_clone());

        Ok(Self {
            series,
            cumulative,
            last_cumulative,
        })
    }

    #[inline(always)]
    pub fn push_block(&mut self, row: C::Row<T>) {
        let len = self.cumulative.len();
        let mut cumulative = match &self.last_cumulative {
            Some((cached_len, row)) if *cached_len == len => row.clone(),
            _ => self
                .cumulative
                .collect_last()
                .unwrap_or_else(|| C::from_fn(|_| T::default())),
        };
        for &column in C::ALL {
            *column.get_mut(&mut cumulative) += *column.get(&row);
        }
        self.cumulative.push(cumulative.clone());
        self.last_cumulative = Some((len + 1, cumulative));
    }

    pub fn reset(&mut self) -> Result<()> {
        self.last_cumulative = None;
        self.cumulative.reset().map_err(Into::into)
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.last_cumulative = None;
        &mut self.cumulative
    }

    pub fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.cumulative
            .validate_and_truncate(version, height)
            .map_err(Into::into)
    }

    pub fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.cumulative
            .truncate_if_needed_at(len)
            .map_err(Into::into)
    }

    pub fn write(&mut self) -> Result<()> {
        self.cumulative.write().map(|_| ()).map_err(Into::into)
    }

    /// Computes one cumulative matrix from one scalar per-block source per column.
    pub fn compute_columns<'a, V, U>(
        &mut self,
        max_from: Height,
        source: impl Fn(C) -> &'a V,
        exit: &Exit,
    ) -> Result<()>
    where
        V: ReadableVec<Height, U> + 'a,
        U: VecValue + Into<T>,
    {
        let dependency_version =
            Version::combine_all(C::ALL.iter().map(|&column| source(column).version()));
        let source_end = C::ALL
            .iter()
            .map(|&column| source(column).len())
            .min()
            .unwrap_or_default();

        self.cumulative
            .validate_computed_version_or_reset(dependency_version)?;
        self.cumulative.truncate_if_needed(max_from)?;
        self.last_cumulative = None;
        let mut last_cumulative = None;
        self.cumulative.repeat_until_complete(exit, |target| {
            let start = target.len();
            let end = target.batch_end(source_end);
            if start >= end {
                return Ok(());
            }

            let mut batches: Vec<_> = C::ALL
                .iter()
                .map(|&column| source(column).collect_range_at(start, end).into_iter())
                .collect();
            let mut cumulative = last_cumulative.take().unwrap_or_else(|| {
                target
                    .collect_last()
                    .unwrap_or_else(|| C::from_fn(|_| T::default()))
            });
            for _ in start..end {
                for (&column, batch) in C::ALL.iter().zip(&mut batches) {
                    *column.get_mut(&mut cumulative) +=
                        batch.next().expect("complete source batch").into();
                }
                target.push(cumulative.clone());
            }
            last_cumulative = Some(cumulative);

            Ok(())
        })?;

        let len = self.cumulative.len();
        self.last_cumulative = last_cumulative
            .or_else(|| self.cumulative.collect_last())
            .map(|row| (len, row));
        Ok(())
    }

    /// Computes one cumulative matrix from two scalar sources per column.
    pub fn compute_columns2<'a, A, B, V1, V2>(
        &mut self,
        max_from: Height,
        source1: impl Fn(C) -> &'a V1,
        source2: impl Fn(C) -> &'a V2,
        mut transform: impl FnMut(C, A, B) -> T,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        V1: ReadableVec<Height, A> + 'a,
        V2: ReadableVec<Height, B> + 'a,
    {
        let dependency_version = Version::combine_all(
            C::ALL
                .iter()
                .flat_map(|&column| [source1(column).version(), source2(column).version()]),
        );
        let source_end = C::ALL
            .iter()
            .flat_map(|&column| [source1(column).len(), source2(column).len()])
            .min()
            .unwrap_or_default();

        self.cumulative
            .validate_computed_version_or_reset(dependency_version)?;
        self.cumulative.truncate_if_needed(max_from)?;
        self.last_cumulative = None;
        self.cumulative.repeat_until_complete(exit, |target| {
            let start = target.len();
            let end = target.batch_end(source_end);
            if start >= end {
                return Ok(());
            }

            let source1_batches: Vec<_> = C::ALL
                .iter()
                .map(|&column| source1(column).collect_range_at(start, end))
                .collect();
            let source2_batches: Vec<_> = C::ALL
                .iter()
                .map(|&column| source2(column).collect_range_at(start, end))
                .collect();
            for offset in 0..(end - start) {
                target.push(C::from_fn(|column| {
                    transform(
                        column,
                        source1_batches[column.index()][offset].clone(),
                        source2_batches[column.index()][offset].clone(),
                    )
                }));
            }

            Ok(())
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Height, StoredU64, Version};
    use vecdb::{
        AnyStoredVec, ColumnId, Database, EagerVec, Exit, ImportableVec, PcoVec, ReadableVec,
        VecValue, WritableVec,
    };

    use super::ColumnarPerBlockCumulativeRolling;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Column {
        A,
        B,
    }

    impl ColumnId for Column {
        type Row<T>
            = [T; 2]
        where
            T: VecValue;

        const VERSION: Version = Version::ONE;
        const ALL: &'static [Self] = &[Self::A, Self::B];

        fn index(self) -> usize {
            self as usize
        }

        fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
            &row[self.index()]
        }

        fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
            &mut row[self.index()]
        }

        fn from_fn<T, F>(mut f: F) -> Self::Row<T>
        where
            T: VecValue,
            F: FnMut(Self) -> T,
        {
            [f(Self::A), f(Self::B)]
        }

        fn map<T, U, F>(row: Self::Row<T>, mut f: F) -> Self::Row<U>
        where
            T: VecValue,
            U: VecValue,
            F: FnMut(T) -> U,
        {
            let [a, b] = row;
            [f(a), f(b)]
        }
    }

    #[test]
    fn pushes_delta_rows_and_recovers_after_truncation() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "brk-columnar-cumulative-{}-{suffix}",
            std::process::id()
        ));
        let db = Database::open(&path).unwrap();
        let mut vec = ColumnarPerBlockCumulativeRolling::<StoredU64, Column, _>::forced_import(
            &db,
            "values",
            Version::ONE,
            |_| (),
        )
        .unwrap();

        vec.push_block([1_u64.into(), 2_u64.into()]);
        vec.push_block([3_u64.into(), 4_u64.into()]);
        assert_eq!(
            vec.cumulative.collect_range_at(0, 2),
            [[1_u64.into(), 2_u64.into()], [4_u64.into(), 6_u64.into()]]
        );

        vec.truncate_if_needed_at(1).unwrap();
        vec.push_block([10_u64.into(), 20_u64.into()]);
        assert_eq!(
            vec.cumulative.collect_one(Height::new(1)),
            Some([11_u64.into(), 22_u64.into()])
        );

        vec.write().unwrap();
        let mut a: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "a", Version::ONE).unwrap();
        let mut b: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "b", Version::ONE).unwrap();
        for value in [1_u64, 3] {
            a.push(value.into());
        }
        for value in [2_u64, 4] {
            b.push(value.into());
        }
        a.write().unwrap();
        b.write().unwrap();

        vec.compute_columns(
            Height::ZERO,
            |column| match column {
                Column::A => &a,
                Column::B => &b,
            },
            &Exit::new(),
        )
        .unwrap();
        assert_eq!(
            vec.cumulative.collect_range_at(0, 2),
            [
                [StoredU64::from(1_u64), StoredU64::from(2_u64)],
                [StoredU64::from(4_u64), StoredU64::from(6_u64)],
            ]
        );

        drop(vec);
        drop(b);
        drop(a);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}

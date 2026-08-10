use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnId, ColumnarVec, Database, EagerVec, Exit, ImportableVec, PcoVec,
    PcoVecValue, ReadOnlyClone, ReadOnlyColumnarVec, ReadableVec, Rw, StorageMode, VecValue,
    WritableVec,
};

#[derive(Deref, DerefMut, Traversable)]
pub struct ColumnarPerBlock<T, C, S: Clone, M: StorageMode = Rw>
where
    T: PcoVecValue,
    C: ColumnId,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: S,
    pub height: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, T>, C>>>,
}

impl<T, C, S: Clone> ColumnarPerBlock<T, C, S>
where
    T: PcoVecValue,
    C: ColumnId,
{
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        build_series: impl FnOnce(&ReadOnlyColumnarVec<PcoVec<Height, T>, C>) -> S,
    ) -> Result<Self> {
        let height = EagerVec::forced_import(db, name, version)?;
        let series = build_series(&height.read_only_clone());

        Ok(Self { series, height })
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, row: C::Row<T>) {
        self.height.push(row);
    }

    pub(crate) fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.height
            .validate_and_truncate(version, height)
            .map_err(Into::into)
    }

    pub(crate) fn validate_computed_version_or_reset(&mut self, version: Version) -> Result<()> {
        self.height
            .validate_computed_version_or_reset(version)
            .map_err(Into::into)
    }

    pub(crate) fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.height.truncate_if_needed_at(len).map_err(Into::into)
    }

    pub(crate) fn write(&mut self) -> Result<()> {
        self.height.write().map(|_| ()).map_err(Into::into)
    }

    pub(crate) fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.height
    }

    /// Computes one stored matrix from two scalar sources per column.
    pub(crate) fn compute_columns2<'a, A, B, V1, V2>(
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
        let dependency_version = C::ALL
            .iter()
            .flat_map(|&column| [source1(column).version(), source2(column).version()])
            .sum();
        let source_end = C::ALL
            .iter()
            .flat_map(|&column| [source1(column).len(), source2(column).len()])
            .min()
            .unwrap_or_default();

        self.height
            .validate_computed_version_or_reset(dependency_version)?;
        self.height.truncate_if_needed(max_from)?;
        self.height.repeat_until_complete(exit, |target| {
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

    /// Computes one stored matrix from a matrix source and one scalar source per column.
    pub(crate) fn compute_matrix_columns2<'a, A, B, V1, V2>(
        &mut self,
        max_from: Height,
        source1: &V1,
        source2: impl Fn(C) -> &'a V2,
        mut transform: impl FnMut(C, A, B) -> T,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        V1: ReadableVec<Height, C::Row<A>>,
        V2: ReadableVec<Height, B> + 'a,
    {
        let dependency_version = C::ALL
            .iter()
            .map(|&column| source2(column).version())
            .sum::<Version>()
            + source1.version();
        let source_end = C::ALL
            .iter()
            .map(|&column| source2(column).len())
            .chain(std::iter::once(source1.len()))
            .min()
            .unwrap_or_default();

        self.height
            .validate_computed_version_or_reset(dependency_version)?;
        self.height.truncate_if_needed(max_from)?;
        self.height.repeat_until_complete(exit, |target| {
            let start = target.len();
            let end = target.batch_end(source_end);
            if start >= end {
                return Ok(());
            }

            let source1_rows = source1.collect_range_at(start, end);
            let source2_batches: Vec<_> = C::ALL
                .iter()
                .map(|&column| source2(column).collect_range_at(start, end))
                .collect();
            for (offset, source1_row) in source1_rows.into_iter().enumerate() {
                target.push(C::from_fn(|column| {
                    transform(
                        column,
                        column.get(&source1_row).clone(),
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
        AnyStoredVec, ColumnId, Database, EagerVec, Exit, ImportableVec, PcoVec, ReadOnlyClone,
        ReadableVec, VecValue, WritableVec,
    };

    use super::ColumnarPerBlock;

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
    fn computes_from_scalar_columns_and_matrix_rows() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "brk-columnar-compute-{}-{suffix}",
            std::process::id()
        ));
        let db = Database::open(&path).unwrap();

        let mut left_a: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "left_a", Version::ONE).unwrap();
        let mut left_b: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "left_b", Version::ONE).unwrap();
        let mut right_a: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "right_a", Version::ONE).unwrap();
        let mut right_b: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "right_b", Version::ONE).unwrap();

        for value in [1_u64, 2, 3] {
            left_a.push(value.into());
        }
        for value in [4_u64, 5] {
            left_b.push(value.into());
        }
        for value in [10_u64, 20, 30] {
            right_a.push(value.into());
        }
        for value in [40_u64, 50, 60] {
            right_b.push(value.into());
        }
        left_a.write().unwrap();
        left_b.write().unwrap();
        right_a.write().unwrap();
        right_b.write().unwrap();

        let mut sums = ColumnarPerBlock::<StoredU64, Column, _>::forced_import(
            &db,
            "sums",
            Version::ONE,
            |_| (),
        )
        .unwrap();
        sums.compute_columns2(
            Height::ZERO,
            |column| match column {
                Column::A => &left_a,
                Column::B => &left_b,
            },
            |column| match column {
                Column::A => &right_a,
                Column::B => &right_b,
            },
            |_, left, right| StoredU64::from(u64::from(left) + u64::from(right)),
            &Exit::new(),
        )
        .unwrap();
        assert_eq!(
            sums.height.collect_range_at(0, 3),
            [
                [StoredU64::from(11_u64), StoredU64::from(44_u64)],
                [StoredU64::from(22_u64), StoredU64::from(55_u64)],
            ]
        );

        let mut factor_a: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "factor_a", Version::ONE).unwrap();
        let mut factor_b: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "factor_b", Version::ONE).unwrap();
        for value in [2_u64, 3] {
            factor_a.push(value.into());
        }
        for value in [5_u64, 7] {
            factor_b.push(value.into());
        }
        factor_a.write().unwrap();
        factor_b.write().unwrap();

        let sums = sums.height.read_only_clone();
        let mut products = ColumnarPerBlock::<StoredU64, Column, _>::forced_import(
            &db,
            "products",
            Version::ONE,
            |_| (),
        )
        .unwrap();
        products
            .compute_matrix_columns2(
                Height::ZERO,
                &sums,
                |column| match column {
                    Column::A => &factor_a,
                    Column::B => &factor_b,
                },
                |_, value, factor| StoredU64::from(u64::from(value) * u64::from(factor)),
                &Exit::new(),
            )
            .unwrap();
        assert_eq!(
            products.height.collect_range_at(0, 3),
            [
                [StoredU64::from(22_u64), StoredU64::from(220_u64)],
                [StoredU64::from(66_u64), StoredU64::from(385_u64)],
            ]
        );

        drop(products);
        drop(sums);
        drop(factor_b);
        drop(factor_a);
        drop(right_b);
        drop(right_a);
        drop(left_b);
        drop(left_a);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}

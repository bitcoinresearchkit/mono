use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, StoredU64, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnId, Database, LazyVec, PcoVec, ReadOnlyClone, ReadOnlyColumnarVec,
    ReadableBoxedVec, ReadableCloneableVec, ReadableColumnarVec, Rw, StorageMode, UnaryTransform,
    VecValue,
};

use crate::internal::{
    CACHE_BUDGET, ColumnarPerBlockCumulativeRolling, StoredU64ToCents, StoredU64ToSats,
};

#[derive(Deref, DerefMut, Traversable)]
pub struct ColumnarValuePerBlockCumulativeRolling<C, S: Clone, M: StorageMode = Rw>
where
    C: ColumnId,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: S,
    /// Reported in satoshis.
    pub sats: ColumnarPerBlockCumulativeRolling<StoredU64, C, (), M>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: ColumnarPerBlockCumulativeRolling<StoredU64, C, (), M>,
}

impl<C, S: Clone> ColumnarValuePerBlockCumulativeRolling<C, S>
where
    C: ColumnId,
{
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        build_series: impl FnOnce(
            &ReadOnlyColumnarVec<PcoVec<Height, StoredU64>, C>,
            &ReadOnlyColumnarVec<PcoVec<Height, StoredU64>, C>,
        ) -> S,
    ) -> Result<Self> {
        let sats = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            &format!("{name}_sats"),
            version,
            |_| (),
        )?;
        let cents = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            &format!("{name}_cents"),
            version,
            |_| (),
        )?;
        let series = build_series(
            &sats.cumulative.read_only_clone(),
            &cents.cumulative.read_only_clone(),
        );

        Ok(Self {
            series,
            sats,
            cents,
        })
    }

    pub(crate) fn sources(
        &self,
        name: &str,
        version: Version,
        columns: impl IntoIterator<Item = C>,
    ) -> (
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    ) {
        Self::sources_from(
            &self.sats.cumulative.read_only_clone(),
            &self.cents.cumulative.read_only_clone(),
            name,
            version,
            columns,
        )
    }

    pub(crate) fn sources_from(
        sats: &ReadOnlyColumnarVec<PcoVec<Height, StoredU64>, C>,
        cents: &ReadOnlyColumnarVec<PcoVec<Height, StoredU64>, C>,
        name: &str,
        version: Version,
        columns: impl IntoIterator<Item = C>,
    ) -> (
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    ) {
        let columns: Box<[_]> = columns.into_iter().collect();
        let sats = Self::typed_source::<StoredU64ToSats, Sats>(
            sats,
            &format!("{name}_sats"),
            version,
            &columns,
        );
        let cents = Self::typed_source::<StoredU64ToCents, Cents>(
            cents,
            &format!("{name}_cents"),
            version,
            &columns,
        );
        (sats, cents)
    }

    fn typed_source<F, T>(
        source: &ReadOnlyColumnarVec<PcoVec<Height, StoredU64>, C>,
        name: &str,
        version: Version,
        columns: &[C],
    ) -> ReadableBoxedVec<Height, T>
    where
        F: UnaryTransform<StoredU64, T>,
        T: VecValue,
    {
        let raw = if columns.len() == 1 {
            source
                .column(name, version, columns[0])
                .read_only_boxed_clone()
        } else {
            source
                .sum_columns(name, version, columns.iter().copied())
                .read_only_boxed_clone()
        };
        let source = LazyVec::transformed::<F>(name, version, raw);
        if columns.len() > 1 {
            CACHE_BUDGET.wrap(source).read_only_boxed_clone()
        } else {
            source.read_only_boxed_clone()
        }
    }

    #[inline(always)]
    pub(crate) fn push_block(&mut self, sats: C::Row<Sats>, cents: C::Row<Cents>) {
        self.sats
            .push_block(C::map(sats, |value| StoredU64::from(u64::from(value))));
        self.cents
            .push_block(C::map(cents, |value| StoredU64::from(u64::from(value))));
    }

    pub(crate) fn len(&self) -> usize {
        self.sats.cumulative.len().min(self.cents.cumulative.len())
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let Self { sats, cents, .. } = self;
        vec![sats.stored_mut(), cents.stored_mut()]
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs, process,
        time::{SystemTime, UNIX_EPOCH},
    };

    use brk_types::{Cents, Sats, Version};
    use vecdb::{AnyVec, ColumnId, Database, VecValue};

    use super::ColumnarValuePerBlockCumulativeRolling;

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

        fn from_fn<T, F>(mut create: F) -> Self::Row<T>
        where
            T: VecValue,
            F: FnMut(Self) -> T,
        {
            [create(Self::A), create(Self::B)]
        }

        fn map<T, U, F>(row: Self::Row<T>, mut map: F) -> Self::Row<U>
        where
            T: VecValue,
            U: VecValue,
            F: FnMut(T) -> U,
        {
            let [a, b] = row;
            [map(a), map(b)]
        }
    }

    #[test]
    fn stores_units_in_separate_cohort_matrices() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "brk-columnar-value-cumulative-{}-{suffix}",
            process::id()
        ));
        let db = Database::open(&path).unwrap();
        let mut values = ColumnarValuePerBlockCumulativeRolling::<Column, _>::forced_import(
            &db,
            "values",
            Version::ONE,
            |_, _| (),
        )
        .unwrap();

        assert_eq!(values.sats.cumulative.name(), "values_sats");
        assert_eq!(values.cents.cumulative.name(), "values_cents");

        values.push_block(
            [Sats::new(1), Sats::new(2)],
            [Cents::new(10), Cents::new(20)],
        );
        values.push_block(
            [Sats::new(3), Sats::new(4)],
            [Cents::new(30), Cents::new(40)],
        );
        values.sats.write().unwrap();
        values.cents.write().unwrap();

        let (sats, cents) = values.sources("values_cumulative", Version::ONE, [Column::A]);
        assert_eq!(sats.collect_range_dyn(0, 2), [Sats::new(1), Sats::new(4)]);
        assert_eq!(
            cents.collect_range_dyn(0, 2),
            [Cents::new(10), Cents::new(40)]
        );

        let (sats, cents) = values.sources(
            "all_values_cumulative",
            Version::ONE,
            Column::ALL.iter().copied(),
        );
        assert_eq!(sats.collect_range_dyn(0, 2), [Sats::new(3), Sats::new(10)]);
        assert_eq!(
            cents.collect_range_dyn(0, 2),
            [Cents::new(30), Cents::new(100)]
        );

        drop(values);
        drop(db);
        fs::remove_dir_all(path).unwrap();
    }
}

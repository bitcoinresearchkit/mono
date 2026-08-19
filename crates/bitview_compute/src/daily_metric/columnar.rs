use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Day1, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, ColumnId, ColumnarVec, Database, EagerVec, ImportableVec, PcoVec, PcoVecValue,
    ReadOnlyClone, ReadOnlyColumnarVec, Rw, StorageMode, WritableVec,
};

use super::DailyValue;

#[derive(Deref, DerefMut, Traversable)]
pub struct ColumnarDailyMetric<T, C, S: Clone, M: StorageMode = Rw>
where
    T: DailyValue + PcoVecValue,
    C: ColumnId,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: S,
    #[traversable(hidden)]
    pub day1: M::Stored<EagerVec<ColumnarVec<PcoVec<Day1, T>, C>>>,
}

impl<T, C, S: Clone> ColumnarDailyMetric<T, C, S>
where
    T: DailyValue + PcoVecValue,
    C: ColumnId,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        build_series: impl FnOnce(&ReadOnlyColumnarVec<PcoVec<Day1, T>, C>) -> S,
    ) -> Result<Self> {
        let day1 = EagerVec::forced_import(db, name, version)?;
        let series = build_series(&day1.read_only_clone());

        Ok(Self { series, day1 })
    }

    #[inline(always)]
    pub fn push(&mut self, row: C::Row<T>) {
        self.day1.push(row);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.day1
    }
}

use bitview_traversable::Traversable;
use brk_types::{Day1, Version};
use vecdb::{
    ColumnId, LazyColumnVec, PcoVec, PcoVecValue, ReadOnlyColumnarVec, ReadableCloneableVec,
    ReadableColumnarVec,
};

use super::{DailyMappings, DailyValue, DailyViews};

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyColumnDailyMetric<T, C>
where
    T: DailyValue + PcoVecValue,
    C: ColumnId,
{
    pub day1: LazyColumnVec<ReadOnlyColumnarVec<PcoVec<Day1, T>, C>, C>,
    #[traversable(flatten)]
    pub views: Box<DailyViews<T>>,
}

impl<T, C> LazyColumnDailyMetric<T, C>
where
    T: DailyValue + PcoVecValue,
    C: ColumnId,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Day1, T>, C>,
        column: C,
        mappings: &DailyMappings,
    ) -> Self {
        let day1 = source.column(name, version, column);
        let views = Box::new(DailyViews::new(
            name,
            day1.read_only_boxed_clone(),
            version,
            mappings,
        ));

        Self { day1, views }
    }
}

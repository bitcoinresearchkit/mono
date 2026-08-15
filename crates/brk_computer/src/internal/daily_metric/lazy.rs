use brk_traversable::Traversable;
use brk_types::{Day1, Version};
use vecdb::{LazyVec, ReadableBoxedVec, ReadableCloneableVec, UnaryTransform, VecValue};

use super::{DailyMappings, DailyValue, DailyViews};

type LazyDay<T, S> = LazyVec<Day1, T, Day1, S>;

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyDailyMetric<T, S>
where
    T: DailyValue,
    S: VecValue,
{
    pub day1: LazyDay<T, S>,
    #[traversable(flatten)]
    pub views: Box<DailyViews<T>>,
}

impl<T, S> LazyDailyMetric<T, S>
where
    T: DailyValue,
    S: VecValue,
{
    pub fn from_source<F>(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Day1, S>,
        mappings: &DailyMappings,
    ) -> Self
    where
        F: UnaryTransform<S, T>,
    {
        let day1 = LazyVec::transformed::<F>(name, version, source);
        let views = Box::new(DailyViews::new(
            name,
            day1.read_only_boxed_clone(),
            version,
            mappings,
        ));

        Self { day1, views }
    }
}

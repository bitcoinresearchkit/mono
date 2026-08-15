use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Day1, Version};
use vecdb::{
    Database, EagerVec, ImportableVec, PcoVec, PcoVecValue, ReadableCloneableVec, Rw, StorageMode,
};

use super::{DailyMappings, DailyValue, DailyViews};

type StoredDay<T, M> = <M as StorageMode>::Stored<EagerVec<PcoVec<Day1, T>>>;

#[derive(Traversable)]
#[traversable(merge)]
pub struct DailyMetric<T, M: StorageMode = Rw>
where
    T: DailyValue + PcoVecValue,
{
    pub day1: StoredDay<T, M>,
    #[traversable(flatten)]
    pub views: Box<DailyViews<T>>,
}

impl<T> DailyMetric<T>
where
    T: DailyValue + PcoVecValue,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        let day1 = EagerVec::forced_import(db, name, version)?;
        let source = day1.read_only_boxed_clone();
        let views = Box::new(DailyViews::new(name, source, version, mappings));

        Ok(Self { day1, views })
    }
}

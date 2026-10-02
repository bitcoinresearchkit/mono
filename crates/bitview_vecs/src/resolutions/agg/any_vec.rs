use vecdb::{AnyVec, VecIndex, VecValue, Version, short_type_name};

use super::LazyAggVec;

impl<I, O, S1I, S1T, Strat> AnyVec for LazyAggVec<I, O, S1I, S1T, Strat>
where
    I: VecIndex,
    O: VecValue,
    S1I: VecIndex,
    S1T: VecValue,
    Strat: 'static,
{
    fn version(&self) -> Version {
        self.version + self.source.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn len(&self) -> usize {
        self.mapping.read().len()
    }

    #[inline]
    fn value_type_to_size_of(&self) -> usize {
        size_of::<O>()
    }

    #[inline]
    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<O>()
    }

    #[inline]
    fn region_names(&self) -> Vec<String> {
        vec![]
    }
}

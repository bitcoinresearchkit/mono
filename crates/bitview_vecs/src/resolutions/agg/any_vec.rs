use vecdb::{AnyVec, VecIndex, VecValue, Version};

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

    fn len(&self) -> usize {
        self.mapping.load().len()
    }

    vecdb::any_vec_type_methods!(I, O);
}

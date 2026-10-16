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
        self.version + self.source.version() + self.mapping.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.mapping.len()
    }

    fn is_mutable(&self) -> bool {
        self.source.is_mutable()
    }

    vecdb::any_vec_type_methods!(I, O);
}

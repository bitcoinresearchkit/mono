use crate::{ReadOnlyClone, VecIndex, VecValue};

use super::LazyVec;

impl<I, T, S> ReadOnlyClone for LazyVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: VecValue,
{
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

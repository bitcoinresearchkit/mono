use super::ReadOnlyCompressedVec;
use crate::{ReadOnlyClone, VecValue, cache::CachePolicy};

pub mod any_vec;
pub mod readable;
pub mod typed;

/// Already read-only: a clone shares the stored vector's publication state.
impl<I: Clone, T: VecValue, S: Clone, C: CachePolicy + Clone> ReadOnlyClone
    for ReadOnlyCompressedVec<I, T, S, C>
{
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

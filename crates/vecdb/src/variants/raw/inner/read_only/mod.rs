use rawdb::Region;

use super::{RawStrategy, ReadOnlyRawVec};
use crate::{VecIndex, VecReader, VecValue, cache::CachePolicy};

pub mod any_vec;
pub mod readable;
pub mod typed;

impl<I, T, S, C: CachePolicy> ReadOnlyRawVec<I, T, S, C>
where
    I: VecIndex,
    T: VecValue,
    S: RawStrategy<T>,
{
    pub fn reader(&self) -> VecReader<I, T, S> {
        VecReader::from_read_only(self)
    }

    pub(crate) fn region(&self) -> &Region {
        self.base.region()
    }
    pub(crate) fn stored_len(&self) -> usize {
        self.base.stored_len()
    }
}

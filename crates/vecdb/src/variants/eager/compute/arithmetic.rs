use brk_exit::Exit;

use super::super::EagerVec;
use crate::{CheckedSub, ReadableVec, Result, StoredVec};

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    pub fn compute_subtract(
        &mut self,
        max_from: V::I,
        subtracted: &impl ReadableVec<V::I, V::T>,
        subtracter: &impl ReadableVec<V::I, V::T>,
        exit: &Exit,
    ) -> Result<()>
    where
        V::T: CheckedSub,
    {
        self.compute_transform2(
            max_from,
            subtracted,
            subtracter,
            |(i, v1, v2, ..)| {
                (
                    i,
                    v1.checked_sub(v2)
                        .expect("subtraction underflow in compute_subtract"),
                )
            },
            exit,
        )
    }
}

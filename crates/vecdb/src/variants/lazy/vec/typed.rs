use crate::{TypedVec, VecIndex, VecValue};

use super::LazyVec;

impl<I, T, S> TypedVec for LazyVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: VecValue,
{
    type I = I;
    type T = T;
}

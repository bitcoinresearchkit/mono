use crate::{Bytes, VecValue};

pub trait BytesVecValue
where
    Self: VecValue + Bytes,
{
}

impl<T> BytesVecValue for T where T: VecValue + Bytes {}

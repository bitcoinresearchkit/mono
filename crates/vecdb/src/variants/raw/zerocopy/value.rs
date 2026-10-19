use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::VecValue;

pub trait ZeroCopyVecValue
where
    Self: VecValue + FromBytes + IntoBytes + Immutable + KnownLayout,
{
}

impl<T> ZeroCopyVecValue for T where T: VecValue + FromBytes + IntoBytes + Immutable + KnownLayout {}

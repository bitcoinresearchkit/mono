#[cfg(feature = "storage")]
use vecdb::VecIndex;

use crate::{
    Day1, Day3, EmptyOutputIndex, Epoch, Halving, Hour1, Hour4, Hour12, Minute10, Minute30, Month1,
    Month3, Month6, OpReturnIndex, StoredU8, Week1, Year, Year1, Year10,
};

#[cfg(feature = "storage")]
impl VecIndex for Day1 {}
#[cfg(feature = "storage")]
impl VecIndex for Day3 {}
#[cfg(feature = "storage")]
impl VecIndex for EmptyOutputIndex {}
#[cfg(feature = "storage")]
impl VecIndex for Epoch {}
#[cfg(feature = "storage")]
impl VecIndex for Halving {}
#[cfg(feature = "storage")]
impl VecIndex for Hour1 {}
#[cfg(feature = "storage")]
impl VecIndex for Hour4 {}
#[cfg(feature = "storage")]
impl VecIndex for Hour12 {}
#[cfg(feature = "storage")]
impl VecIndex for Minute10 {}
#[cfg(feature = "storage")]
impl VecIndex for Minute30 {}
#[cfg(feature = "storage")]
impl VecIndex for Month1 {}
#[cfg(feature = "storage")]
impl VecIndex for Month3 {}
#[cfg(feature = "storage")]
impl VecIndex for Month6 {}
#[cfg(feature = "storage")]
impl VecIndex for OpReturnIndex {}
#[cfg(feature = "storage")]
impl VecIndex for StoredU8 {}
#[cfg(feature = "storage")]
impl VecIndex for Week1 {}
#[cfg(feature = "storage")]
impl VecIndex for Year {}
#[cfg(feature = "storage")]
impl VecIndex for Year1 {}
#[cfg(feature = "storage")]
impl VecIndex for Year10 {}

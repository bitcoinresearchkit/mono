#[cfg(feature = "storage")]
use vecdb::VecIndex;

use crate::{Day1, Day3, Epoch, Halving, Month1, Month3, Month6, Week1, Year, Year1, Year10};

#[cfg(feature = "storage")]
impl VecIndex for Day1 {}
#[cfg(feature = "storage")]
impl VecIndex for Day3 {}
#[cfg(feature = "storage")]
impl VecIndex for Epoch {}
#[cfg(feature = "storage")]
impl VecIndex for Halving {}
#[cfg(feature = "storage")]
impl VecIndex for Month1 {}
#[cfg(feature = "storage")]
impl VecIndex for Month3 {}
#[cfg(feature = "storage")]
impl VecIndex for Month6 {}
#[cfg(feature = "storage")]
impl VecIndex for Week1 {}
#[cfg(feature = "storage")]
impl VecIndex for Year {}
#[cfg(feature = "storage")]
impl VecIndex for Year1 {}
#[cfg(feature = "storage")]
impl VecIndex for Year10 {}

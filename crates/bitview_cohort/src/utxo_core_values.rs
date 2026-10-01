use std::ops::AddAssign;

use crate::{AgeRange, ByEpoch, Class};

/// Values for disjoint UTXO age, epoch, and class cohorts.
#[derive(Clone, Default)]
pub struct UTXOCoreValues<T> {
    pub age_range: AgeRange<T>,
    pub epoch: ByEpoch<T>,
    pub class: Class<T>,
}

impl<T> UTXOCoreValues<T> {
    pub fn map<U>(&self, mut map: impl FnMut(&T) -> U) -> UTXOCoreValues<U> {
        UTXOCoreValues {
            age_range: AgeRange::from_fn(|id| map(id.select(&self.age_range))),
            epoch: ByEpoch::from_fn(|id| map(id.select(&self.epoch))),
            class: Class::from_fn(|id| map(id.select(&self.class))),
        }
    }
}

impl<T: AddAssign + Copy> AddAssign for UTXOCoreValues<T> {
    fn add_assign(&mut self, rhs: Self) {
        for (left, right) in self.age_range.iter_mut().zip(rhs.age_range.iter()) {
            *left += *right;
        }
        for (left, right) in self.epoch.iter_mut().zip(rhs.epoch.iter()) {
            *left += *right;
        }
        for (left, right) in self.class.iter_mut().zip(rhs.class.iter()) {
            *left += *right;
        }
    }
}

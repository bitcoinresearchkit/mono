use std::ops::AddAssign;

use crate::{AgeRange, ByEpoch, Class, CohortId, UTXOValues};

/// Values for disjoint UTXO age, epoch, and class cohorts.
#[derive(Clone, Default)]
pub struct UTXOCoreValues<T> {
    pub age_range: AgeRange<T>,
    pub epoch: ByEpoch<T>,
    pub class: Class<T>,
}

impl<T> UTXOCoreValues<T> {
    /// Resolve a direct cohort or sum its disjoint age ranges before storing it.
    pub fn value(&self, id: CohortId) -> Option<T>
    where
        T: Copy + AddAssign,
    {
        match id {
            CohortId::Epoch(epoch) => Some(*epoch.select(&self.epoch)),
            CohortId::Class(class) => Some(*class.select(&self.class)),
            CohortId::Age(range) => Some(*range.select(&self.age_range)),
            _ => {
                let mut ranges = id.age_ranges()?;
                let mut total = *ranges.next()?.select(&self.age_range);
                for id in ranges {
                    total += *id.select(&self.age_range);
                }
                Some(total)
            }
        }
    }

    pub fn map<U>(&self, mut map: impl FnMut(&T) -> U) -> UTXOCoreValues<U> {
        UTXOCoreValues {
            age_range: AgeRange::from_fn(|id| map(id.select(&self.age_range))),
            epoch: ByEpoch::from_fn(|id| map(id.select(&self.epoch))),
            class: Class::from_fn(|id| map(id.select(&self.class))),
        }
    }
}

impl<T> From<UTXOValues<T>> for UTXOCoreValues<T> {
    fn from(cohort_values: UTXOValues<T>) -> Self {
        cohort_values.core
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

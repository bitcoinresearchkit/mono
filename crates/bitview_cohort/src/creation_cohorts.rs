use std::ops::AddAssign;

use crate::{AgeRange, ByEpoch, Class, CohortId};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

/// Disjoint age, creation-epoch and creation-year cohorts.
#[derive(Default, Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct CreationCohorts<T> {
    pub age: AgeRange<T>,
    pub epoch: ByEpoch<T>,
    pub class: Class<T>,
}

impl<T> CreationCohorts<T> {
    pub fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            age: AgeRange::try_new(&mut create)?,
            epoch: ByEpoch::try_new(&mut create)?,
            class: Class::try_new(create)?,
        })
    }

    pub fn new(mut create: impl FnMut(CohortId) -> T) -> Self {
        Self {
            age: AgeRange::new(&mut create),
            epoch: ByEpoch::new(&mut create),
            class: Class::new(create),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.age
            .iter()
            .chain(self.epoch.iter())
            .chain(self.class.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.age
            .iter_mut()
            .chain(self.epoch.iter_mut())
            .chain(self.class.iter_mut())
    }

    pub fn get(&self, id: CohortId) -> Option<&T> {
        match id {
            CohortId::Age(id) => Some(id.select(&self.age)),
            CohortId::Epoch(id) => Some(id.select(&self.epoch)),
            CohortId::Class(id) => Some(id.select(&self.class)),
            _ => None,
        }
    }

    pub fn map_with_id<U>(&self, mut map: impl FnMut(CohortId, &T) -> U) -> CreationCohorts<U> {
        CreationCohorts {
            age: AgeRange::from_fn(|id| map(id.cohort(), id.select(&self.age))),
            epoch: ByEpoch::from_fn(|id| map(id.cohort(), id.select(&self.epoch))),
            class: Class::from_fn(|id| map(id.cohort(), id.select(&self.class))),
        }
    }

    pub fn map<U>(&self, mut map: impl FnMut(&T) -> U) -> CreationCohorts<U> {
        self.map_with_id(|_, value| map(value))
    }
}

impl<T: AddAssign + Copy> AddAssign for CreationCohorts<T> {
    fn add_assign(&mut self, rhs: Self) {
        for (left, right) in self.iter_mut().zip(rhs.iter()) {
            *left += *right;
        }
    }
}

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

use crate::{AgeRange, ByEpoch, Class, CohortId};

#[derive(Default, Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct UTXOGroupCore<T> {
    /// Uses all UTXOs.
    pub all: T,
    pub age: AgeRange<T>,
    pub epoch: ByEpoch<T>,
    pub class: Class<T>,
}

impl<T> UTXOGroupCore<T> {
    pub fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            all: create(CohortId::All)?,
            age: AgeRange::try_new(&mut create)?,
            epoch: ByEpoch::try_new(&mut create)?,
            class: Class::try_new(&mut create)?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.all]
            .into_iter()
            .chain(self.age.iter())
            .chain(self.epoch.iter())
            .chain(self.class.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.all]
            .into_iter()
            .chain(self.age.iter_mut())
            .chain(self.epoch.iter_mut())
            .chain(self.class.iter_mut())
    }

    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(CohortId) -> T,
    {
        Self {
            all: create(CohortId::All),
            age: AgeRange::new(&mut create),
            epoch: ByEpoch::new(&mut create),
            class: Class::new(&mut create),
        }
    }

    pub fn get(&self, id: CohortId) -> Option<&T> {
        match id {
            CohortId::All => Some(&self.all),
            CohortId::Age(age) => Some(age.select(&self.age)),
            CohortId::Epoch(epoch) => Some(epoch.select(&self.epoch)),
            CohortId::Class(class) => Some(class.select(&self.class)),
            CohortId::Term(_) | CohortId::Amount(_) | CohortId::Type(_) | CohortId::Entry(_) => {
                None
            }
        }
    }

    pub fn map_with_id<U>(&self, mut map: impl FnMut(CohortId, &T) -> U) -> UTXOGroupCore<U> {
        UTXOGroupCore {
            all: map(CohortId::All, &self.all),
            age: AgeRange::from_fn(|id| map(id.cohort(), id.select(&self.age))),
            epoch: ByEpoch::from_fn(|id| map(id.cohort(), id.select(&self.epoch))),
            class: Class::from_fn(|id| map(id.cohort(), id.select(&self.class))),
        }
    }
}

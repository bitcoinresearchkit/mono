use crate::{CohortId, CreationCohorts, UtxoGroups};

/// A fixed family of disjoint cohorts, abstracted over what each cohort holds,
/// so per-cohort storage is written once for every family.
pub trait CohortGroup {
    type Of<T>;

    fn new<T>(create: impl FnMut(CohortId) -> T) -> Self::Of<T>;
    fn try_new<T, E>(create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self::Of<T>, E>;
    /// `None` for cohorts outside this group.
    fn get<T>(cohorts: &Self::Of<T>, id: CohortId) -> Option<&T>;
    fn iter<'a, T: 'a>(cohorts: &'a Self::Of<T>) -> impl Iterator<Item = &'a T>;
    fn iter_mut<'a, T: 'a>(cohorts: &'a mut Self::Of<T>) -> impl Iterator<Item = &'a mut T>;
    fn map<T, U>(cohorts: &Self::Of<T>, map: impl FnMut(&T) -> U) -> Self::Of<U>;
}

/// Age, creation-epoch and creation-year cohorts.
pub struct Creation;

impl CohortGroup for Creation {
    type Of<T> = CreationCohorts<T>;

    fn new<T>(create: impl FnMut(CohortId) -> T) -> Self::Of<T> {
        CreationCohorts::new(create)
    }
    fn try_new<T, E>(create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self::Of<T>, E> {
        CreationCohorts::try_new(create)
    }
    fn get<T>(cohorts: &Self::Of<T>, id: CohortId) -> Option<&T> {
        cohorts.get(id)
    }
    fn iter<'a, T: 'a>(cohorts: &'a Self::Of<T>) -> impl Iterator<Item = &'a T> {
        cohorts.iter()
    }
    fn iter_mut<'a, T: 'a>(cohorts: &'a mut Self::Of<T>) -> impl Iterator<Item = &'a mut T> {
        cohorts.iter_mut()
    }
    fn map<T, U>(cohorts: &Self::Of<T>, map: impl FnMut(&T) -> U) -> Self::Of<U> {
        cohorts.map(map)
    }
}

/// Amount-range and spendable-type cohorts.
pub struct Utxo;

impl CohortGroup for Utxo {
    type Of<T> = UtxoGroups<T>;

    fn new<T>(create: impl FnMut(CohortId) -> T) -> Self::Of<T> {
        UtxoGroups::new(create)
    }
    fn try_new<T, E>(create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self::Of<T>, E> {
        UtxoGroups::try_new(create)
    }
    fn get<T>(cohorts: &Self::Of<T>, id: CohortId) -> Option<&T> {
        cohorts.get(id)
    }
    fn iter<'a, T: 'a>(cohorts: &'a Self::Of<T>) -> impl Iterator<Item = &'a T> {
        cohorts.iter()
    }
    fn iter_mut<'a, T: 'a>(cohorts: &'a mut Self::Of<T>) -> impl Iterator<Item = &'a mut T> {
        cohorts.iter_mut()
    }
    fn map<T, U>(cohorts: &Self::Of<T>, map: impl FnMut(&T) -> U) -> Self::Of<U> {
        cohorts.map(map)
    }
}

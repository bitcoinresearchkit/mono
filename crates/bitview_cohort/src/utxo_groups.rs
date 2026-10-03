use std::ops::AddAssign;

use crate::{AmountRange, CohortId, SpendableType, SpendableTypeId};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

/// Disjoint UTXO amount-range and spendable-type cohorts.
#[derive(Default, Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct UtxoGroups<T> {
    pub utxo_amount: AmountRange<T>,
    pub type_: SpendableType<T>,
}

impl<T> UtxoGroups<T> {
    pub fn new(mut create: impl FnMut(CohortId) -> T) -> Self {
        Self {
            utxo_amount: AmountRange::new(&mut create),
            type_: SpendableType::new(create),
        }
    }

    pub fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            utxo_amount: AmountRange::try_new(&mut create)?,
            type_: SpendableType::try_new(create)?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.utxo_amount.iter().chain(self.type_.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.utxo_amount.iter_mut().chain(self.type_.iter_mut())
    }

    pub fn get(&self, id: CohortId) -> Option<&T> {
        match id {
            CohortId::Amount(id) => Some(id.select(&self.utxo_amount)),
            CohortId::Type(output_type) => {
                SpendableTypeId::from_output_type(output_type).map(|id| id.select(&self.type_))
            }
            _ => None,
        }
    }

    pub fn map_with_id<U>(&self, mut map: impl FnMut(CohortId, &T) -> U) -> UtxoGroups<U> {
        UtxoGroups {
            utxo_amount: AmountRange::from_fn(|id| map(id.cohort(), id.select(&self.utxo_amount))),
            type_: self.type_.map_with_id(map),
        }
    }

    pub fn map<U>(&self, mut map: impl FnMut(&T) -> U) -> UtxoGroups<U> {
        self.map_with_id(|_, value| map(value))
    }
}

impl<T: AddAssign + Copy> AddAssign for UtxoGroups<T> {
    fn add_assign(&mut self, rhs: Self) {
        for (left, right) in self.iter_mut().zip(rhs.iter()) {
            *left += *right;
        }
    }
}

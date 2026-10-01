use derive_more::{Deref, DerefMut};
use rayon::prelude::*;

use crate::{AmountRange, ByTerm, CohortId, SpendableType, SpendableTypeId, UTXOGroupCore};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

#[derive(Default, Clone, Deref, DerefMut)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct UTXOGroups<T> {
    #[deref]
    #[deref_mut]
    #[cfg_attr(feature = "storage", traversable(flatten))]
    pub core: UTXOGroupCore<T>,
    /// Groups UTXOs by their individual output value.
    pub utxo_amount: AmountRange<T>,
    pub term: ByTerm<T>,
    #[cfg_attr(feature = "storage", traversable(rename = "type"))]
    pub type_: SpendableType<T>,
}

impl<T> UTXOGroups<T> {
    pub fn get(&self, id: CohortId) -> Option<&T> {
        match id {
            CohortId::Term(term) => Some(self.term.get(term)),
            CohortId::Amount(amount) => Some(amount.select(&self.utxo_amount)),
            CohortId::Type(kind) => {
                SpendableTypeId::from_output_type(kind).map(|kind| kind.select(&self.type_))
            }
            _ => self.core.get(id),
        }
    }

    pub fn map_with_id<U>(&self, mut map: impl FnMut(CohortId, &T) -> U) -> UTXOGroups<U> {
        UTXOGroups {
            core: self.core.map_with_id(&mut map),
            utxo_amount: AmountRange::from_fn(|id| map(id.cohort(), id.select(&self.utxo_amount))),
            term: self.term.map_with_id(&mut map),
            type_: self.type_.map_with_id(map),
        }
    }

    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(CohortId) -> T,
    {
        Self {
            core: UTXOGroupCore::new(&mut create),
            utxo_amount: AmountRange::new(&mut create),
            term: ByTerm::new(&mut create),
            type_: SpendableType::new(&mut create),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.core.all]
            .into_iter()
            .chain(self.term.iter())
            .chain(self.utxo_amount.iter())
            .chain(self.core.age.iter())
            .chain(self.core.epoch.iter())
            .chain(self.core.class.iter())
            .chain(self.type_.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.core.all]
            .into_iter()
            .chain(self.term.iter_mut())
            .chain(self.utxo_amount.iter_mut())
            .chain(self.core.age.iter_mut())
            .chain(self.core.epoch.iter_mut())
            .chain(self.core.class.iter_mut())
            .chain(self.type_.iter_mut())
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [&mut self.core.all]
            .into_par_iter()
            .chain(self.term.par_iter_mut())
            .chain(self.utxo_amount.par_iter_mut())
            .chain(self.core.age.par_iter_mut())
            .chain(self.core.epoch.par_iter_mut())
            .chain(self.core.class.par_iter_mut())
            .chain(self.type_.par_iter_mut())
    }
}

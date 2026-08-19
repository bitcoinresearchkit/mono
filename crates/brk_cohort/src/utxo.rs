use bitview_traversable::Traversable;
use rayon::prelude::*;

use crate::{
    Amount, ByAge, ByEntry, ByEpoch, ByTerm, CLASS_FILTERS, CLASS_NAMES, Class, ClassId,
    ENTRY_FILTERS, ENTRY_NAMES, EPOCH_FILTERS, EPOCH_NAMES, EntryId, EpochId, Filter,
    SPENDABLE_TYPE_FILTERS, SPENDABLE_TYPE_NAMES, SpendableType, TERM_FILTERS, TERM_NAMES,
};
use vecdb::ColumnId;

#[derive(Default, Clone, Traversable)]
pub struct UTXOGroups<T> {
    pub all: T,
    pub age: ByAge<T>,
    pub epoch: ByEpoch<T>,
    pub class: Class<T>,
    pub entry: ByEntry<T>,
    pub utxo_amount: Amount<T>,
    pub term: ByTerm<T>,
    #[traversable(rename = "type")]
    pub type_: SpendableType<T>,
}

impl<T> UTXOGroups<T> {
    pub fn get(&self, filter: &Filter) -> Option<&T> {
        match filter {
            Filter::All => Some(&self.all),
            Filter::Term(term) => match term {
                crate::Term::Sth => Some(&self.term.short),
                crate::Term::Lth => Some(&self.term.long),
            },
            Filter::Time(_) => self.age.get(filter),
            Filter::Amount(_) => self.utxo_amount.get(filter),
            Filter::Epoch(_) => EpochId::ALL
                .iter()
                .copied()
                .find(|id| id.select(&EPOCH_FILTERS) == filter)
                .map(|id| id.select(&self.epoch)),
            Filter::Class(_) => ClassId::ALL
                .iter()
                .copied()
                .find(|id| id.select(&CLASS_FILTERS) == filter)
                .map(|id| id.select(&self.class)),
            Filter::Entry(_) => EntryId::ALL
                .iter()
                .copied()
                .find(|id| id.select(&ENTRY_FILTERS) == filter)
                .map(|id| id.select(&self.entry)),
            Filter::Type(output_type) => Some(self.type_.get(*output_type)),
        }
    }

    pub fn map_named<U>(
        &self,
        mut map: impl FnMut(&Filter, &'static str, &T) -> U,
    ) -> UTXOGroups<U> {
        UTXOGroups {
            all: map(&Filter::All, "", &self.all),
            age: self.age.map_named(&mut map),
            epoch: ByEpoch::from_fn(|id| {
                map(
                    id.select(&EPOCH_FILTERS),
                    id.select(&EPOCH_NAMES).id,
                    id.select(&self.epoch),
                )
            }),
            class: Class::from_fn(|id| {
                map(
                    id.select(&CLASS_FILTERS),
                    id.select(&CLASS_NAMES).id,
                    id.select(&self.class),
                )
            }),
            entry: ByEntry::from_fn(|id| {
                map(
                    id.select(&ENTRY_FILTERS),
                    id.select(&ENTRY_NAMES).id,
                    id.select(&self.entry),
                )
            }),
            utxo_amount: self.utxo_amount.map_named(&mut map),
            term: ByTerm::from_fn(|id| {
                map(
                    id.select(&TERM_FILTERS),
                    id.select(&TERM_NAMES).id,
                    id.select(&self.term),
                )
            }),
            type_: SpendableType::from_fn(|id| {
                map(
                    id.select(&SPENDABLE_TYPE_FILTERS),
                    id.select(&SPENDABLE_TYPE_NAMES).id,
                    id.select(&self.type_),
                )
            }),
        }
    }

    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter, &'static str) -> T,
    {
        Self {
            all: create(Filter::All, ""),
            age: ByAge::new(&mut create),
            epoch: ByEpoch::new(&mut create),
            class: Class::new(&mut create),
            entry: ByEntry::new(&mut create),
            utxo_amount: Amount::new(&mut create),
            term: ByTerm::new(&mut create),
            type_: SpendableType::new(&mut create),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.all]
            .into_iter()
            .chain(self.term.iter())
            .chain(self.age.under.iter())
            .chain(self.age.over.iter())
            .chain(self.utxo_amount.iter())
            .chain(self.age.range.iter())
            .chain(self.epoch.iter())
            .chain(self.class.iter())
            .chain(self.entry.iter())
            .chain(self.type_.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.all]
            .into_iter()
            .chain(self.term.iter_mut())
            .chain(self.age.under.iter_mut())
            .chain(self.age.over.iter_mut())
            .chain(self.utxo_amount.iter_mut())
            .chain(self.age.range.iter_mut())
            .chain(self.epoch.iter_mut())
            .chain(self.class.iter_mut())
            .chain(self.entry.iter_mut())
            .chain(self.type_.iter_mut())
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [&mut self.all]
            .into_par_iter()
            .chain(self.term.par_iter_mut())
            .chain(self.age.under.par_iter_mut())
            .chain(self.age.over.par_iter_mut())
            .chain(self.utxo_amount.par_iter_mut())
            .chain(self.age.range.par_iter_mut())
            .chain(self.epoch.par_iter_mut())
            .chain(self.class.par_iter_mut())
            .chain(self.entry.par_iter_mut())
            .chain(self.type_.par_iter_mut())
    }

    pub fn iter_separate(&self) -> impl Iterator<Item = &T> {
        self.age
            .range
            .iter()
            .chain(self.epoch.iter())
            .chain(self.class.iter())
            .chain(self.entry.iter())
            .chain(self.utxo_amount.range.iter())
            .chain(self.type_.iter())
    }

    pub fn iter_separate_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.age
            .range
            .iter_mut()
            .chain(self.epoch.iter_mut())
            .chain(self.class.iter_mut())
            .chain(self.entry.iter_mut())
            .chain(self.utxo_amount.range.iter_mut())
            .chain(self.type_.iter_mut())
    }

    pub fn par_iter_separate_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        self.age
            .range
            .par_iter_mut()
            .chain(self.epoch.par_iter_mut())
            .chain(self.class.par_iter_mut())
            .chain(self.entry.par_iter_mut())
            .chain(self.utxo_amount.range.par_iter_mut())
            .chain(self.type_.par_iter_mut())
    }

    pub fn iter_overlapping_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.all]
            .into_iter()
            .chain(self.term.iter_mut())
            .chain(self.age.under.iter_mut())
            .chain(self.age.over.iter_mut())
            .chain(self.utxo_amount.under.iter_mut())
            .chain(self.utxo_amount.over.iter_mut())
    }

    /// Iterator over aggregate cohorts (all, sth, lth) that compute values from sub-cohorts.
    /// These are cohorts with StateLevel::PriceOnly that derive values from stateful sub-cohorts.
    pub fn iter_aggregate(&self) -> impl Iterator<Item = &T> {
        [&self.all].into_iter().chain(self.term.iter())
    }

    pub fn par_iter_aggregate(&self) -> impl ParallelIterator<Item = &T>
    where
        T: Send + Sync,
    {
        [&self.all].into_par_iter().chain(self.term.par_iter())
    }

    /// Iterator over aggregate cohorts (all, sth, lth) that compute values from sub-cohorts.
    /// These are cohorts with StateLevel::PriceOnly that derive values from stateful sub-cohorts.
    pub fn iter_aggregate_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.all].into_iter().chain(self.term.iter_mut())
    }

    pub fn par_iter_aggregate_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [&mut self.all]
            .into_par_iter()
            .chain(self.term.par_iter_mut())
    }
}

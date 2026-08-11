use brk_traversable::Traversable;
use vecdb::ColumnId;

use crate::{
    ByAge, ByEntry, ByEpoch, ByTerm, CLASS_FILTERS, CLASS_NAMES, Class, ClassId, ENTRY_FILTERS,
    ENTRY_NAMES, EPOCH_FILTERS, EPOCH_NAMES, EntryId, EpochId, Filter, SPENDABLE_TYPE_FILTERS,
    SPENDABLE_TYPE_NAMES, SpendableType, TERM_FILTERS, TERM_NAMES,
};

#[derive(Default, Clone, Traversable)]
pub struct UTXOGroupsWithoutAmount<T> {
    pub all: T,
    pub age: ByAge<T>,
    pub epoch: ByEpoch<T>,
    pub class: Class<T>,
    pub entry: ByEntry<T>,
    pub term: ByTerm<T>,
    #[traversable(rename = "type")]
    pub type_: SpendableType<T>,
}

impl<T> UTXOGroupsWithoutAmount<T> {
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
            term: ByTerm::new(&mut create),
            type_: SpendableType::new(&mut create),
        }
    }

    pub fn get(&self, filter: &Filter) -> Option<&T> {
        match filter {
            Filter::All => Some(&self.all),
            Filter::Term(term) => match term {
                crate::Term::Sth => Some(&self.term.short),
                crate::Term::Lth => Some(&self.term.long),
            },
            Filter::Time(_) => self.age.get(filter),
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
            Filter::Amount(_) => None,
        }
    }

    pub fn map_named<U>(
        &self,
        mut map: impl FnMut(&Filter, &'static str, &T) -> U,
    ) -> UTXOGroupsWithoutAmount<U> {
        UTXOGroupsWithoutAmount {
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
}

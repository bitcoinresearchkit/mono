use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;

use crate::Filter;

#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct UTXOAllAndSth<T> {
    pub all: T,
    pub sth: T,
}

define_column_id!(
    UTXOAllAndSthId for UTXOAllAndSth, version = 1 {
        All => all,
        Sth => sth,
    }
);

impl<T> UTXOAllAndSth<T> {
    pub fn get(&self, filter: &Filter) -> Option<&T> {
        match filter {
            Filter::All => Some(&self.all),
            Filter::Term(crate::Term::Sth) => Some(&self.sth),
            _ => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.all, &self.sth].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.all, &mut self.sth].into_iter()
    }
}

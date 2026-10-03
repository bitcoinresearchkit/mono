use std::sync::Arc;

use bitview_primitives::Index;

use super::SeriesEntry;

#[derive(Default)]
pub(super) struct Series<'a> {
    entries: Vec<SeriesEntry<'a>>,
    pub(super) description: Option<Arc<str>>,
}

impl<'a> Series<'a> {
    pub(super) fn insert(&mut self, entry: SeriesEntry<'a>) -> Option<SeriesEntry<'a>> {
        match self
            .entries
            .binary_search_by_key(&entry.index(), |entry| entry.index())
        {
            Ok(position) => {
                let previous = self.entries[position];
                self.entries[position] = entry;
                Some(previous)
            }
            Err(position) => {
                self.entries.insert(position, entry);
                None
            }
        }
    }

    pub(super) fn get(&self, index: Index) -> Option<&SeriesEntry<'a>> {
        self.entries
            .binary_search_by_key(&index, |entry| entry.index())
            .ok()
            .map(|position| &self.entries[position])
    }

    pub(super) fn indexes(&self) -> impl Iterator<Item = Index> + '_ {
        self.entries.iter().map(|entry| entry.index())
    }

    pub(super) fn first(&self) -> Option<&SeriesEntry<'a>> {
        self.entries.first()
    }
}

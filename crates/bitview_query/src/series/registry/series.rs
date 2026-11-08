use std::sync::Arc;

use bitview_primitives::Index;

use super::SeriesEntry;

#[derive(Default)]
pub(super) struct Series<'a> {
    entries: Vec<SeriesEntry<'a>>,
    pub(super) description: Option<Arc<str>>,
}

impl<'a> Series<'a> {
    /// Each index is inserted once.
    pub(super) fn insert(&mut self, entry: SeriesEntry<'a>) {
        let position = self.entries.partition_point(|e| e.index < entry.index);
        debug_assert!(
            self.entries
                .get(position)
                .is_none_or(|e| e.index != entry.index),
            "Duplicate series index {:?}",
            entry.index
        );
        self.entries.insert(position, entry);
    }

    pub(super) fn get(&self, index: Index) -> Option<&SeriesEntry<'a>> {
        self.entries
            .binary_search_by_key(&index, |entry| entry.index)
            .ok()
            .map(|position| &self.entries[position])
    }

    pub(super) fn indexes(&self) -> impl Iterator<Item = Index> + '_ {
        self.entries.iter().map(|entry| entry.index)
    }

    /// The indexes whose values can be null.
    pub(super) fn nullable(&self) -> impl Iterator<Item = Index> + '_ {
        self.entries
            .iter()
            .filter(|entry| entry.vec.nullable())
            .map(|entry| entry.index)
    }

    pub(super) fn first(&self) -> Option<&SeriesEntry<'a>> {
        self.entries.first()
    }
}

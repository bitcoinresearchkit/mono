use std::sync::Arc;

use bitview_primitives::{Accumulation, Index};

use super::SeriesEntry;

pub(super) struct Series<'a> {
    entries: Vec<SeriesEntry<'a>>,
    pub(super) description: Option<Arc<str>>,
    /// Its metadata's, set at the registry's build.
    pub(super) accumulation: Accumulation,
    /// Whether its metadata says its values depend on the price, whatever their type.
    pub(super) priced: bool,
}

impl Default for Series<'_> {
    /// Before its metadata, which a series may not have: the undeclared accumulation.
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            description: None,
            accumulation: Accumulation::UNDECLARED,
            priced: false,
        }
    }
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

    /// On a chain without a price, every value that depends on it is null, never made up from
    /// the missing price (a fiat type always does; others say so in their metadata). An index
    /// whose type has no null serves no rows.
    pub(super) fn leave_prices_out(&mut self) {
        for entry in &mut self.entries {
            let fiat = matches!(
                entry.vec.value_type_name(),
                "Dollars" | "Cents" | "CentsSigned" | "CentsFract" | "OHLCDollars" | "OHLCCents"
            );
            if self.priced || fiat {
                entry.null_values = entry.vec.nullable();
                entry.available &= entry.null_values;
            }
        }
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

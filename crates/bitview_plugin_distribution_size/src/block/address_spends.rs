use std::collections::hash_map::Entry;

use bitview_cohort::ByAddrType;
use brk_types::{Cents, Height, OutputType, Sats, TypeIndex};
use rustc_hash::FxHashMap;

/// Origin groups in the established hash order, with input order within each group.
/// Dense rows and links avoid allocating a separate vector for every origin.
pub struct AddressSpends {
    origins: FxHashMap<Height, (u32, u32)>,
    rows: Vec<(OutputType, TypeIndex, Sats)>,
    next: Vec<u32>,
}

impl AddressSpends {
    pub fn new(input_count: usize) -> Self {
        Self {
            origins: FxHashMap::with_capacity_and_hasher(
                (input_count / 4).max(16),
                Default::default(),
            ),
            rows: Vec::with_capacity(input_count),
            next: Vec::with_capacity(input_count),
        }
    }

    pub fn push(&mut self, height: Height, ty: OutputType, index: TypeIndex, value: Sats) {
        let position = u32::try_from(self.rows.len() + 1).expect("too many block inputs") - 1;
        self.rows.push((ty, index, value));
        self.next.push(u32::MAX);
        match self.origins.entry(height) {
            Entry::Vacant(entry) => {
                entry.insert((position, position));
            }
            Entry::Occupied(mut entry) => {
                let (_, tail) = entry.get_mut();
                self.next[*tail as usize] = position;
                *tail = position;
            }
        }
    }

    pub fn into_typed(self, prices: &[Cents]) -> ByAddrType<Vec<(TypeIndex, Sats, Cents)>> {
        let mut typed = ByAddrType::<Vec<(TypeIndex, Sats, Cents)>>::default();
        for (height, (mut position, _)) in self.origins {
            let price = prices[usize::from(height)];
            while position != u32::MAX {
                let (ty, index, value) = self.rows[position as usize];
                typed.get_mut_unwrap(ty).push((index, value, price));
                position = self.next[position as usize];
            }
        }
        typed
    }
}

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap},
    path::Path,
};

use brk_cohort::{AgeRangeId, CohortContext, Filter, TERM_NAMES, UTXO_ALL_NAME, UTXOAggregate};
use brk_error::Result;
use brk_types::{CentsCompact, Date, Sats, UrpdRaw};
use rayon::prelude::*;
use vecdb::ColumnId;

use super::{COST_BASIS_PRICE_DIGITS, UTXOStates};

impl UTXOStates {
    pub fn write_urpds(&self, date: Date, states_path: &Path, sth_filter: &Filter) -> Result<()> {
        AgeRangeId::ALL
            .iter()
            .map(|&id| (id, id.select(&self.age_range)))
            .collect::<Vec<_>>()
            .into_par_iter()
            .try_for_each(|(id, state)| -> Result<()> {
                let mut merged = Vec::<(CentsCompact, Sats)>::new();
                for (&price, &sats) in state.cost_basis_map() {
                    let rounded = rounded_price(price);
                    if let Some(last) = merged.last_mut()
                        && last.0 == rounded
                    {
                        last.1 += sats;
                    } else {
                        merged.push((rounded, sats));
                    }
                }
                let name = CohortContext::Utxo.prefixed(id.name().id);
                UrpdRaw::write(states_path, &name, date, merged.into_iter())
            })?;

        let maps: Vec<_> = AgeRangeId::ALL
            .iter()
            .filter_map(|&id| {
                let map = id.select(&self.age_range).cost_basis_map();
                (!map.is_empty()).then(|| (map, sth_filter.includes(id.filter())))
            })
            .collect();
        if maps.is_empty() {
            return Ok(());
        }

        let capacity = maps.iter().map(|(map, _)| map.len()).max().unwrap_or(0);
        let mut targets = UTXOAggregate {
            all: MergeTarget::new(capacity),
            sth: MergeTarget::new(capacity),
            lth: MergeTarget::new(capacity),
        };
        merge_k_way(&maps, &mut targets);

        [
            (UTXO_ALL_NAME.id, targets.all.merged),
            (TERM_NAMES.short.id, targets.sth.merged),
            (TERM_NAMES.long.id, targets.lth.merged),
        ]
        .into_par_iter()
        .try_for_each(|(name, merged)| UrpdRaw::write(states_path, name, date, merged.into_iter()))
    }

    pub fn age_range_urpd_entries(
        &self,
    ) -> impl Iterator<Item = (AgeRangeId, CentsCompact, Sats)> + '_ {
        AgeRangeId::ALL.iter().copied().flat_map(move |id| {
            id.select(&self.age_range)
                .cost_basis_map()
                .iter()
                .map(move |(&price, &sats)| (id, rounded_price(price), sats))
        })
    }
}

#[inline]
fn rounded_price(price: CentsCompact) -> CentsCompact {
    price.round_to_dollar(COST_BASIS_PRICE_DIGITS)
}

struct MergeTarget {
    price_sats: u64,
    merged: Vec<(CentsCompact, Sats)>,
}

impl MergeTarget {
    fn new(capacity: usize) -> Self {
        Self {
            price_sats: 0,
            merged: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    fn accumulate(&mut self, amount: u64) {
        self.price_sats += amount;
    }

    fn finalize_price(&mut self, price: CentsCompact) {
        if self.price_sats > 0 {
            let rounded = rounded_price(price);
            if let Some((last_price, last_sats)) = self.merged.last_mut()
                && *last_price == rounded
            {
                *last_sats += Sats::from(self.price_sats);
            } else {
                self.merged.push((rounded, Sats::from(self.price_sats)));
            }
        }
        self.price_sats = 0;
    }
}

fn merge_k_way(
    maps: &[(&BTreeMap<CentsCompact, Sats>, bool)],
    targets: &mut UTXOAggregate<MergeTarget>,
) {
    let mut iterators: Vec<_> = maps
        .iter()
        .map(|(map, is_sth)| (map.iter().peekable(), *is_sth))
        .collect();
    let mut heap = BinaryHeap::<Reverse<(CentsCompact, usize)>>::with_capacity(iterators.len());

    for (index, (iterator, _)) in iterators.iter_mut().enumerate() {
        if let Some(&(&price, _)) = iterator.peek() {
            heap.push(Reverse((price, index)));
        }
    }

    let mut current_price = None;
    while let Some(Reverse((price, index))) = heap.pop() {
        let (iterator, is_sth) = &mut iterators[index];
        let (_, &sats) = iterator.next().unwrap();

        if let Some(previous) = current_price
            && previous != price
        {
            targets
                .iter_mut()
                .for_each(|target| target.finalize_price(previous));
        }

        current_price = Some(price);
        let amount = u64::from(sats);
        targets.all.accumulate(amount);
        if *is_sth {
            targets.sth.accumulate(amount);
        } else {
            targets.lth.accumulate(amount);
        }

        if let Some(&(&next_price, _)) = iterator.peek() {
            heap.push(Reverse((next_price, index)));
        }
    }

    if let Some(price) = current_price {
        targets
            .iter_mut()
            .for_each(|target| target.finalize_price(price));
    }
}

use std::{cmp::Ordering, collections::BTreeMap, path::Path};

use bitview_cohort::{
    AgeRangeId, CohortContext, STH_AGE_RANGE_COUNT, TERM_NAMES, UTXO_ALL_NAME, UTXOAggregate,
};
use brk_error::Result;
use brk_types::{CentsCompact, Date, Sats, UrpdRaw};
use rayon::prelude::*;
use vecdb::ColumnId;

use super::{COST_BASIS_PRICE_DIGITS, UTXOStates};

impl UTXOStates {
    pub fn write_urpds(&self, date: Date, states_path: &Path) -> Result<()> {
        let mut urpds: Vec<_> = AgeRangeId::ALL
            .par_iter()
            .map(|&id| DailyUrpd::from_age_range(id, id.select(&self.age_range).cost_basis_map()))
            .collect();

        if urpds.iter().any(|urpd| !urpd.entries.is_empty()) {
            let (sth_urpds, lth_urpds) = urpds.split_at(STH_AGE_RANGE_COUNT);
            let (sth, lth) = rayon::join(
                || DailyUrpd::merge_group(sth_urpds),
                || DailyUrpd::merge_group(lth_urpds),
            );
            let aggregates = UTXOAggregate {
                all: DailyUrpd::merge_sorted(&sth, &lth),
                sth,
                lth,
            };
            urpds.extend([
                DailyUrpd::from_entries(UTXO_ALL_NAME.id, aggregates.all),
                DailyUrpd::from_entries(TERM_NAMES.short.id, aggregates.sth),
                DailyUrpd::from_entries(TERM_NAMES.long.id, aggregates.lth),
            ]);
        }

        urpds
            .into_par_iter()
            .try_for_each(|urpd| urpd.write(states_path, date))
    }

    pub fn age_range_urpd_entries(
        &self,
    ) -> impl Iterator<Item = (AgeRangeId, CentsCompact, Sats)> + '_ {
        AgeRangeId::ALL.iter().copied().flat_map(move |id| {
            id.select(&self.age_range)
                .cost_basis_map()
                .iter()
                .map(move |(&price, &sats)| (id, DailyUrpd::rounded_price(price), sats))
        })
    }
}

struct DailyUrpd {
    name: String,
    entries: Vec<(CentsCompact, Sats)>,
}

impl DailyUrpd {
    fn from_age_range(id: AgeRangeId, map: &BTreeMap<CentsCompact, Sats>) -> Self {
        let mut entries = Vec::<(CentsCompact, Sats)>::new();
        for (&price, &sats) in map {
            let price = Self::rounded_price(price);
            if let Some(last) = entries.last_mut()
                && last.0 == price
            {
                last.1 += sats;
            } else {
                entries.push((price, sats));
            }
        }

        Self {
            name: CohortContext::Utxo.prefixed(id.name().id),
            entries,
        }
    }

    fn from_entries(name: &str, entries: Vec<(CentsCompact, Sats)>) -> Self {
        Self {
            name: name.to_owned(),
            entries,
        }
    }

    fn merge_group(urpds: &[Self]) -> Vec<(CentsCompact, Sats)> {
        urpds
            .par_iter()
            .filter(|urpd| !urpd.entries.is_empty())
            .map(|urpd| urpd.entries.clone())
            .reduce_with(|left, right| Self::merge_sorted(&left, &right))
            .unwrap_or_default()
    }

    fn merge_sorted(
        left: &[(CentsCompact, Sats)],
        right: &[(CentsCompact, Sats)],
    ) -> Vec<(CentsCompact, Sats)> {
        let mut merged = Vec::with_capacity(left.len() + right.len());
        let mut left_index = 0;
        let mut right_index = 0;

        while left_index < left.len() && right_index < right.len() {
            let left_entry = left[left_index];
            let right_entry = right[right_index];
            match left_entry.0.cmp(&right_entry.0) {
                Ordering::Less => {
                    merged.push(left_entry);
                    left_index += 1;
                }
                Ordering::Greater => {
                    merged.push(right_entry);
                    right_index += 1;
                }
                Ordering::Equal => {
                    merged.push((left_entry.0, left_entry.1 + right_entry.1));
                    left_index += 1;
                    right_index += 1;
                }
            }
        }

        merged.extend_from_slice(&left[left_index..]);
        merged.extend_from_slice(&right[right_index..]);
        merged
    }

    #[inline]
    fn rounded_price(price: CentsCompact) -> CentsCompact {
        price.round_to_dollar(COST_BASIS_PRICE_DIGITS)
    }

    fn write(self, states_path: &Path, date: Date) -> Result<()> {
        UrpdRaw::write(states_path, &self.name, date, self.entries.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_range_rounding_combines_equal_prices() {
        let map = BTreeMap::from([
            (CentsCompact::new(12_345), Sats::from(2_u64)),
            (CentsCompact::new(12_349), Sats::from(3_u64)),
            (CentsCompact::new(12_400), Sats::from(5_u64)),
        ]);

        let urpd = DailyUrpd::from_age_range(AgeRangeId::Under1H, &map);

        assert_eq!(urpd.name, "utxos_under_1h_old");
        assert_eq!(
            urpd.entries,
            vec![
                (CentsCompact::new(12_300), Sats::from(5_u64)),
                (CentsCompact::new(12_400), Sats::from(5_u64)),
            ]
        );
    }

    #[test]
    fn sorted_merge_preserves_order_and_sums_equal_prices() {
        let left = [
            (CentsCompact::new(100), Sats::from(1_u64)),
            (CentsCompact::new(300), Sats::from(3_u64)),
        ];
        let right = [
            (CentsCompact::new(200), Sats::from(2_u64)),
            (CentsCompact::new(300), Sats::from(4_u64)),
            (CentsCompact::new(400), Sats::from(5_u64)),
        ];

        assert_eq!(
            DailyUrpd::merge_sorted(&left, &right),
            vec![
                (CentsCompact::new(100), Sats::from(1_u64)),
                (CentsCompact::new(200), Sats::from(2_u64)),
                (CentsCompact::new(300), Sats::from(7_u64)),
                (CentsCompact::new(400), Sats::from(5_u64)),
            ]
        );
        assert_eq!(DailyUrpd::merge_sorted(&[], &right), right);
        assert_eq!(DailyUrpd::merge_sorted(&left, &[]), left);
    }
}

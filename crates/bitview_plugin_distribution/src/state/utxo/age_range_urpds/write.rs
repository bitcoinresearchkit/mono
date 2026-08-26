use std::{collections::BTreeMap, fs, path::Path};

use bitview_cohort::{AgeRange, AgeRangeId};
use brk_error::Result;
use brk_types::{CentsCompact, Date, Sats, UrpdRaw};
use vecdb::ColumnId;

use super::super::{COST_BASIS_PRICE_DIGITS, UTXOCohortState, UTXOStates};
use super::{AgeRangeUrpds, HEADER_LEN};
use crate::state::{CostBasisData, RealizedState, WithCapital};

impl AgeRangeUrpds {
    fn from_states(
        states: &AgeRange<UTXOCohortState<RealizedState, CostBasisData<WithCapital>>>,
    ) -> Self {
        Self {
            entries: AgeRange::par_from_fn(|id| {
                Self::rounded_entries(id.select(states).cost_basis_map())
            }),
        }
    }

    fn write(&self, states_path: &Path, date: Date) -> Result<()> {
        let sections =
            AgeRange::par_try_from_fn(|id| UrpdRaw::serialize_iter(self.get(id).iter().copied()))?;
        let capacity = HEADER_LEN + sections.iter().map(Vec::len).sum::<usize>();
        let mut buffer = Self::new_buffer(capacity);
        for (index, id) in AgeRangeId::ALL.iter().copied().enumerate() {
            buffer.extend_from_slice(id.select(&sections));
            Self::set_offset(&mut buffer, index + 1);
        }

        fs::create_dir_all(Self::dir(states_path))?;
        fs::write(Self::path(states_path, date), buffer)?;
        Ok(())
    }

    fn rounded_entries(map: &BTreeMap<CentsCompact, Sats>) -> Vec<(CentsCompact, Sats)> {
        let mut entries = Vec::<(CentsCompact, Sats)>::new();
        for (&price, &sats) in map {
            let price = price.round_to_dollar(COST_BASIS_PRICE_DIGITS);
            if let Some(last) = entries.last_mut()
                && last.0 == price
            {
                last.1 += sats;
            } else {
                entries.push((price, sats));
            }
        }
        entries
    }
}

impl UTXOStates {
    pub fn write_urpds(&self, date: Date, states_path: &Path) -> Result<()> {
        AgeRangeUrpds::from_states(&self.age_range).write(states_path, date)
    }

    pub fn age_range_urpd_entries(
        &self,
        id: AgeRangeId,
    ) -> impl Iterator<Item = (CentsCompact, Sats)> + '_ {
        id.select(&self.age_range)
            .cost_basis_map()
            .iter()
            .map(|(&price, &sats)| (price.round_to_dollar(COST_BASIS_PRICE_DIGITS), sats))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bitview_cohort::{AgeRange, AgeRangeId, UTXOAggregateId};
    use brk_types::{CentsCompact, Date, Sats};
    use vecdb::ColumnId;

    use super::AgeRangeUrpds;

    #[test]
    fn packed_file_reads_all_or_one_age_range() {
        let root = tempfile::tempdir().unwrap();
        let date = Date::new(2026, 8, 23);
        let expected = AgeRangeUrpds {
            entries: AgeRange::from_fn(|id| {
                vec![(
                    CentsCompact::new((id.index() as u32 + 1) * 100),
                    Sats::from(id.index() as u64 + 1),
                )]
            }),
        };
        expected.write(root.path(), date).unwrap();

        let actual = AgeRangeUrpds::read(root.path(), date).unwrap();
        for id in AgeRangeId::ALL.iter().copied() {
            assert_eq!(actual.get(id), expected.get(id));
        }

        let id = AgeRangeId::From2YTo3Y;
        let one = AgeRangeUrpds::read_one(root.path(), id, date).unwrap();
        assert_eq!(
            one.map,
            expected.get(id).iter().copied().collect::<BTreeMap<_, _>>()
        );

        for id in UTXOAggregateId::ALL.iter().copied() {
            assert_eq!(
                AgeRangeUrpds::read_aggregate(root.path(), id, date)
                    .unwrap()
                    .map,
                expected.aggregate(id).map
            );
        }
    }
}

use std::path::Path;

use bitview_cohort::AgeRangeId;
use bitview_urpd::{AgeRangeUrpds, prune_snapshots};
use brk_error::Result;
use brk_types::{CentsCompact, Date, Sats};

use super::UTXOStates;

impl UTXOStates {
    pub(crate) fn prepare_urpds(
        &self,
        states_path: &Path,
        previous_date: Option<Date>,
        next_date: Option<Date>,
    ) -> Result<()> {
        prune_snapshots(&AgeRangeUrpds::dir(states_path), previous_date)?;
        if let Some(date) = previous_date.filter(|date| Some(*date) != next_date) {
            self.write_urpds(date, states_path)?;
        }
        Ok(())
    }

    pub fn age_urpds(&self) -> AgeRangeUrpds {
        AgeRangeUrpds::from_sorted_entries(|age| self.age_range_entries(age))
    }

    pub fn write_urpds(&self, date: Date, states_path: &Path) -> Result<()> {
        self.age_urpds().write(states_path, date)
    }

    fn age_range_entries(&self, id: AgeRangeId) -> impl Iterator<Item = (CentsCompact, Sats)> + '_ {
        id.select(&self.age_range)
            .cost_basis_map()
            .iter()
            .map(|(&price, &sats)| (price, sats))
    }
}

#[cfg(test)]
mod tests {
    use bitview_cohort::AgeRange;
    use bitview_urpd::DailyUrpds;
    use brk_types::{Cents, SupplyState};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn live_weights_match_saved_buckets_and_rewind_republishes_the_recovered_day() {
        let root = tempdir().unwrap();
        let mut states = UTXOStates::new(root.path());
        for age in states.age_range.iter_mut() {
            age.reset_cost_basis_data_if_needed().unwrap();
        }
        let age = AgeRangeId::Under1H.select_mut(&mut states.age_range);
        for (price, sats) in [(101, 4_u64), (102, 6)] {
            age.receive_utxo(
                &SupplyState {
                    utxo_count: 1,
                    value: Sats::from(sats),
                },
                Cents::new(price),
            );
        }
        age.apply_pending();
        let dates = [
            Date::new(2026, 9, 1),
            Date::new(2026, 9, 2),
            Date::new(2026, 9, 3),
        ];
        for date in dates {
            states.write_urpds(date, root.path()).unwrap();
        }
        let age_urpds = states.age_urpds();
        let weighted = |current| {
            age_urpds
                .with_entries(root.path(), dates[1], current, |entries| {
                    DailyUrpds::from_age_entries(entries, &AgeRange::from_fn(|_| 0.7)).all
                })
                .unwrap()
                .unwrap()
        };
        assert_eq!(weighted(true), weighted(false));
        assert_eq!(
            weighted(true).as_ref(),
            [(CentsCompact::new(100), Sats::from(7_u64))]
        );

        // Recovered state is the last block of the preceding day. Its saved
        // snapshot may still contain outputs from removed blocks of that day.
        age_receive(&mut states, 5);
        states
            .prepare_urpds(root.path(), Some(dates[1]), Some(dates[2]))
            .unwrap();
        assert!(AgeRangeUrpds::path(root.path(), dates[0]).exists());
        assert!(!AgeRangeUrpds::path(root.path(), dates[2]).exists());
        let recovered = AgeRangeUrpds::read(root.path(), dates[1]).unwrap();
        assert_eq!(
            recovered.get(AgeRangeId::Under1H),
            [(CentsCompact::new(100), Sats::from(15_u64))]
        );
        states
            .prepare_urpds(root.path(), None, Some(dates[0]))
            .unwrap();
        assert!(
            dates
                .iter()
                .all(|date| !AgeRangeUrpds::path(root.path(), *date).exists())
        );
    }

    fn age_receive(states: &mut UTXOStates, sats: u64) {
        let age = AgeRangeId::Under1H.select_mut(&mut states.age_range);
        age.receive_utxo(
            &SupplyState {
                utxo_count: 1,
                value: Sats::from(sats),
            },
            Cents::new(100),
        );
        age.apply_pending();
    }
}

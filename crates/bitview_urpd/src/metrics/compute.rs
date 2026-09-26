use std::{fs, iter};

use bitview_cohort::{AgeRange, UTXOAggregateId};
use bitview_compute::{collect_cohort_weights, prepare_computed};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Date, Day1, Sats, StoredF64, Version};
use vecdb::{ReadableVec, WritableVec};

use super::{Metrics, WRITE_INTERVAL_DAYS};
use crate::{
    COMPUTE_VERSION,
    distribution::{DailyUrpds, PriceStats},
};

impl Metrics {
    #[allow(clippy::too_many_arguments)]
    pub fn compute(
        &mut self,
        version: Version,
        recompute_from: usize,
        dates: &impl ReadableVec<Day1, Date>,
        spot: &impl ReadableVec<Day1, Option<Cents>>,
        weights: &AgeRange<&impl ReadableVec<Day1, Option<StoredF64>>>,
        supplies: &AgeRange<&impl ReadableVec<Day1, Option<Sats>>>,
        mut load: impl FnMut(Day1, Date, &AgeRange<f64>) -> Result<Option<DailyUrpds>>,
        exit: &Exit,
    ) -> Result<()> {
        let version = Version::combine_all(
            iter::once(version + COMPUTE_VERSION)
                .chain(iter::once(dates.version()))
                .chain(iter::once(spot.version()))
                .chain(weights.iter().map(|v| v.version()))
                .chain(supplies.iter().map(|v| v.version())),
        );
        let marker = self.states_path.join("urpd.version");
        let current = marker.try_exists()? && Version::try_from(marker.as_path())? == version;
        let from = if current { recompute_from } else { 0 };
        let end = iter::once(dates.len())
            .chain(iter::once(spot.len()))
            .chain(weights.iter().map(|v| v.len()))
            .chain(supplies.iter().map(|v| v.len()))
            .min()
            .unwrap_or_default();
        let start = prepare_computed(
            self.stored_vecs_mut().collect::<Vec<_>>(),
            version,
            from.min(end),
        )?;
        self.prune_snapshots(start, current)?;
        for index in start..end {
            let day = Day1::from(index);
            let date = dates.collect_one(day);
            let urpds = match (date, collect_cohort_weights(day, weights, supplies)) {
                (Some(date), Some(weights)) => load(day, date, &weights)?,
                _ => None,
            };
            let close = spot.collect_one(day).flatten().unwrap_or(Cents::NAN);
            for &id in UTXOAggregateId::ALL {
                let urpd = urpds.as_ref().map(|u| u.aggregate(id));
                let stats =
                    PriceStats::from_entries(urpd.into_iter().flat_map(|u| u.iter().copied()));
                id.select_mut(&mut self.cost_basis).push(&stats.cost_basis);
                id.select_mut(&mut self.capitalized_price_stored)
                    .push(stats.capitalized_price);
                if let (Some(urpd), Some(date)) = (urpd, date) {
                    self.write_snapshot(id, date, urpd)?;
                }
            }
            self.supply_density.push(urpds.as_ref(), close);
            if (index + 1).is_multiple_of(WRITE_INTERVAL_DAYS) || index + 1 == end {
                let _lock = exit.lock();
                for vec in self.stored_vecs_mut() {
                    vec.write()?;
                }
            }
        }
        fs::create_dir_all(&self.states_path)?;
        let _lock = exit.lock();
        version.write(&marker)?;
        Ok(())
    }
}

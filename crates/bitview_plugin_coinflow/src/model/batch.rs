use bitview_cohort::AgeRange;
use bitview_compute::{
    AgeBand, CohortAccounting, MINIMUM_DURATION_DAYS, collect_age_range, weighted_age_aggregates,
};
use bitview_plugin_age::AccountingSources;
use bitview_primitives::{BoundedRatio, CoinDays, Float64, PerDay};
use brk_types::{Bitcoin, Height, Sats, Timestamp};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use vecdb::ReadableVec;

use super::{PrimaryValues, decay::DecayFit};

#[derive(Default)]
pub(crate) struct PrimaryBatch {
    timestamps: Vec<Timestamp>,
    transfer_volumes: AgeRange<Vec<Sats>>,
    coindays_created: AgeRange<Vec<CoinDays>>,
    accounting: CohortAccounting,
}

impl PrimaryBatch {
    pub(crate) fn collect_into(
        &mut self,
        timestamps: &impl ReadableVec<Height, Timestamp>,
        transfer_volumes: &AgeRange<&impl ReadableVec<Height, Sats>>,
        coindays_created: &AgeRange<&impl ReadableVec<Height, CoinDays>>,
        accounting: &AccountingSources<'_>,
        start: usize,
        end: usize,
    ) {
        timestamps.collect_range_into_at(start, end, &mut self.timestamps);
        collect_age_range(transfer_volumes, &mut self.transfer_volumes, start, end);
        collect_age_range(coindays_created, &mut self.coindays_created, start, end);
        accounting.collect_into(start, end, &mut self.accounting);
    }

    #[inline]
    fn len(&self) -> usize {
        self.timestamps.len()
    }

    pub(crate) fn primary_values_batch(
        &self,
        genesis_timestamp: Timestamp,
        bounds: &AgeRange<AgeBand>,
    ) -> Vec<PrimaryValues> {
        (0..self.len())
            .into_par_iter()
            .map(|offset| self.primary_values(offset, genesis_timestamp, bounds))
            .collect()
    }

    fn primary_values(
        &self,
        offset: usize,
        genesis_timestamp: Timestamp,
        bounds: &AgeRange<AgeBand>,
    ) -> PrimaryValues {
        let hazards = AgeRange::from_fn(|id| {
            Self::spending_rate(
                id.select(&self.transfer_volumes)[offset],
                id.select(&self.coindays_created)[offset],
            )
        });
        let network_age = self.timestamps[offset]
            .difference_in_days_between_float(genesis_timestamp)
            .max(MINIMUM_DURATION_DAYS);
        let exposures = DecayFit::exposures(&hazards, network_age, bounds);
        let mobilities =
            AgeRange::from_fn(|id| BoundedRatio::from(AgeBand::mobility(*id.select(&exposures))));
        let cohorts = weighted_age_aggregates(|id| {
            let mobility = *id.select(&mobilities);
            self.accounting.weighted(id, offset, mobility)
        });

        PrimaryValues {
            spending_rate: AgeRange::from_fn(|id| PerDay::from(*id.select(&hazards))),
            spending_exposure: AgeRange::from_fn(|id| Float64::from(*id.select(&exposures))),
            mobility: mobilities,
            cohorts,
        }
    }

    #[inline]
    fn spending_rate(transfer_volume: Sats, coindays_created: CoinDays) -> f64 {
        let exposure = f64::from(coindays_created);
        if exposure > 0.0 {
            (f64::from(Bitcoin::from(transfer_volume)) / exposure).max(0.0)
        } else {
            0.0
        }
    }
}

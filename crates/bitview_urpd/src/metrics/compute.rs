use std::{iter, mem};

use bitview_cohort::{AgeRange, UTXOAggregateId};
use bitview_compute::{collect_cohort_weights, prepare_computed};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Sats, StoredF64, Version};
use vecdb::{ReadableVec, WritableVec};

use super::{Metrics, WRITE_INTERVAL_BLOCKS, metric_buckets::MetricBuckets};
use crate::{COMPUTE_VERSION, ReplayInputs};

impl Metrics {
    #[allow(clippy::too_many_arguments)]
    pub fn compute(
        &mut self,
        version: Version,
        recompute_from: usize,
        inputs: ReplayInputs<'_>,
        weights: &AgeRange<&impl ReadableVec<Height, StoredF64>>,
        supplies: &AgeRange<&impl ReadableVec<Height, Sats>>,
        exit: &Exit,
    ) -> Result<()> {
        let spot = inputs.prices;
        let version = Version::combine_all(
            iter::once(version + COMPUTE_VERSION)
                .chain(iter::once(spot.version()))
                .chain(weights.iter().map(|v| v.version()))
                .chain(supplies.iter().map(|v| v.version())),
        );
        let end = iter::once(spot.len())
            .chain(weights.iter().map(|v| v.len()))
            .chain(supplies.iter().map(|v| v.len()))
            .min()
            .unwrap_or_default();
        let start = prepare_computed(
            self.stored_vecs_mut().collect::<Vec<_>>(),
            version,
            recompute_from.min(end),
        )?;
        let mut replay = mem::take(&mut self.replay);
        let mut buffer = mem::take(&mut self.buffer);
        replay.for_each(start..end, inputs, |height, close, source| {
            let supplies =
                AgeRange::try_from_fn(|age| age.select(supplies).collect_one(height).ok_or(()))
                    .ok();
            let weights = supplies
                .as_ref()
                .and_then(|s| collect_cohort_weights(height, weights, s));
            if let Some(weights) = &weights {
                buffer.update(source.buckets(), weights, close);
            }
            self.push_block(weights.as_ref().map(|_| &buffer));
            if (usize::from(height) + 1).is_multiple_of(WRITE_INTERVAL_BLOCKS)
                || usize::from(height) + 1 == end
            {
                let _lock = exit.lock();
                for vec in self.stored_vecs_mut() {
                    vec.write()?;
                }
            }
            Ok(())
        })?;
        self.replay = replay;
        self.buffer = buffer;
        Ok(())
    }

    fn push_block(&mut self, buckets: Option<&MetricBuckets>) {
        for &id in UTXOAggregateId::ALL {
            let stats = buckets
                .map(|b| id.select(&b.prices).stats())
                .unwrap_or_default();
            id.select_mut(&mut self.cost_basis).push(&stats.cost_basis);
            id.select_mut(&mut self.capitalized_price_stored)
                .push(stats.capitalized_price);
        }
        self.supply_density.push(buckets.map(|b| &b.density));
    }
}

use std::{array, iter, mem};

use bitview_cohort::{AgeAggregate, AgeAggregateId, AgeRange};
use bitview_compute::{collect_cohort_weights, prepare_computed};
use bitview_primitives::{CostBasisByPercentile, Ratio64};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Sats, Version};
use vecdb::ReadableVec;

use super::{
    Metrics, WRITE_INTERVAL_BLOCKS,
    density::SupplyDensity,
    metric_buckets::{CohortBlock, MetricBuckets},
};
use crate::{COMPUTE_VERSION, ReplayInputs};

impl Metrics {
    #[allow(clippy::too_many_arguments)]
    pub fn compute(
        &mut self,
        version: Version,
        recompute_from: usize,
        inputs: ReplayInputs<'_>,
        weights: &AgeRange<&(impl ReadableVec<Height, Ratio64> + Sync)>,
        supplies: &AgeRange<&(impl ReadableVec<Height, Sats> + Sync)>,
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
            exit,
        )?;
        let mut replay = mem::take(&mut self.replay);
        let cohorts: [_; AgeAggregateId::ALL.len()] =
            array::from_fn(|i| AgeAggregateId::ALL[i].age_range_ids());
        replay.map(
            start..end,
            inputs,
            || Worker {
                supplies: AgeRange::from_fn(|age| age.select(supplies).cursor()),
                weights: AgeRange::from_fn(|age| age.select(weights).cursor()),
                buckets: MetricBuckets::default(),
            },
            |worker, height, close, source| {
                let supplies = AgeRange::try_from_fn(|age| {
                    age.select_mut(&mut worker.supplies)
                        .get(usize::from(height))
                        .ok_or(())
                })
                .ok();
                let Some(weights) = supplies
                    .as_ref()
                    .and_then(|s| collect_cohort_weights(height, &mut worker.weights, s))
                else {
                    return Ok(None);
                };
                worker
                    .buckets
                    .update(source.project(&[Some(&weights)], cohorts), close);
                Ok(Some(worker.buckets.block()))
            },
            |height, block| {
                self.push_block(block.as_ref());
                if (usize::from(height) + 1).is_multiple_of(WRITE_INTERVAL_BLOCKS)
                    || usize::from(height) + 1 == end
                {
                    let _lock = exit.lock();
                    for vec in self.stored_vecs_mut() {
                        vec.write()?;
                    }
                }
                Ok(())
            },
        )?;
        self.replay = replay;
        Ok(())
    }

    fn push_block(&mut self, block: Option<&AgeAggregate<CohortBlock>>) {
        for &id in AgeAggregateId::ALL {
            match block {
                Some(block) => {
                    let block = id.select(block);
                    id.select_mut(&mut self.cohorts)
                        .push(&block.cost_basis, &block.density);
                }
                None => id
                    .select_mut(&mut self.cohorts)
                    .push(&CostBasisByPercentile::default(), &SupplyDensity::NAN),
            }
        }
    }
}

/// One segment's cursors and bucket buffer.
struct Worker<S, W> {
    supplies: AgeRange<S>,
    weights: AgeRange<W>,
    buckets: MetricBuckets,
}

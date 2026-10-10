use std::{array, iter};

use bitview_cohort::{AgeAggregate, AgeAggregateId, AgeRange};
use bitview_compute::{collect_cohort_weights, prepare_computed};
use bitview_primitives::{CostBasisByPercentile, Ratio64};
use bitview_vecs::Density;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Sats, Version};
use vecdb::{AnyStoredVec, ReadableVec};

use super::{
    CostBasisVecs, WRITE_INTERVAL_BLOCKS,
    metric_buckets::{CohortBlock, MetricBuckets},
};
use crate::{COMPUTE_VERSION, Replay, ReplayInputs};

const COHORTS: usize = AgeAggregateId::ALL.len();

/// Replays the weighted URPD over `recompute_from..` and pushes every cohort's cost basis,
/// `cohorts` in `AgeAggregateId::ALL` order.
#[allow(clippy::too_many_arguments)]
pub fn compute_cost_basis(
    replay: &mut Replay,
    mut cohorts: [&mut CostBasisVecs; COHORTS],
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
        stored_vecs_mut(&mut cohorts).collect::<Vec<_>>(),
        version,
        recompute_from.min(end),
        exit,
    )?;
    let age_ranges: [_; COHORTS] = array::from_fn(|i| AgeAggregateId::ALL[i].age_range_ids());
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
                .update(source.project(&[Some(&weights)], age_ranges), close);
            Ok(Some(worker.buckets.block()))
        },
        |height, block| {
            push_block(&mut cohorts, block.as_ref());
            if (usize::from(height) + 1).is_multiple_of(WRITE_INTERVAL_BLOCKS)
                || usize::from(height) + 1 == end
            {
                let _lock = exit.lock();
                for vec in stored_vecs_mut(&mut cohorts) {
                    vec.write()?;
                }
            }
            Ok(())
        },
    )
}

fn push_block(
    cohorts: &mut [&mut CostBasisVecs; COHORTS],
    block: Option<&AgeAggregate<CohortBlock>>,
) {
    for (&id, cohort) in AgeAggregateId::ALL.iter().zip(cohorts.iter_mut()) {
        match block {
            Some(block) => {
                let block = id.select(block);
                cohort.push(
                    &block.cost_basis,
                    &block.supply_density,
                    &block.capital_density,
                );
            }
            None => cohort.push(
                &CostBasisByPercentile::default(),
                &Density::NAN,
                &Density::NAN,
            ),
        }
    }
}

fn stored_vecs_mut<'a>(
    cohorts: &'a mut [&mut CostBasisVecs; COHORTS],
) -> impl Iterator<Item = &'a mut dyn AnyStoredVec> {
    cohorts
        .iter_mut()
        .flat_map(|cohort| cohort.stored_vecs_mut())
}

/// One segment's cursors and bucket buffer.
struct Worker<S, W> {
    supplies: AgeRange<S>,
    weights: AgeRange<W>,
    buckets: MetricBuckets,
}

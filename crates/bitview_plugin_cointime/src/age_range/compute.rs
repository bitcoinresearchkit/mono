use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_compute::{Inputs, collect_age_range, write_chunks, write_locked};
use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_primitives::{BoundedRatio, CoinDays};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Bitcoin, Height, Sats, Version};
use vecdb::{AnyStoredVec, AnyVec, ComputeVec, ReadableVec};

use super::{RangeVecs, Vecs};

const HOURS_PER_DAY: f64 = 24.0;

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, age: &AgeVecs, exit: &Exit) -> Result<()> {
        let starting_height = indexer.compute_start().height;
        let transfer_volumes = AgeRange::from_fn(|id| {
            &id.select(&age.ranges)
                .activity
                .transfer_volume
                .value
                .block
                .sats
        });
        let coindays_destroyed = AgeRange::from_fn(|id| {
            &id.select(&age.ranges)
                .activity
                .coindays_destroyed
                .value
                .block
        });
        let coindays_created =
            AgeRange::from_fn(|id| &id.select(&age.ranges).coindays_created.cumulative.height);

        self.compute_consumed(
            starting_height,
            &transfer_volumes,
            &coindays_destroyed,
            exit,
        )?;
        self.compute_rest(starting_height, &coindays_created, exit)
    }

    fn compute_consumed(
        &mut self,
        starting_height: Height,
        transfer_volumes: &AgeRange<&impl ReadableVec<Height, Sats>>,
        source_coindays_destroyed: &AgeRange<&impl ReadableVec<Height, CoinDays>>,
        exit: &Exit,
    ) -> Result<()> {
        let inputs = Inputs::new(
            Version::ZERO,
            transfer_volumes
                .iter()
                .map(|vec| *vec as &dyn AnyVec)
                .chain(
                    source_coindays_destroyed
                        .iter()
                        .map(|vec| *vec as &dyn AnyVec),
                ),
        );
        let range = inputs.prepare(
            self.consumed_vecs_mut().collect::<Vec<_>>(),
            usize::from(starting_height),
            exit,
        )?;
        let bounds = age_bounds_days();
        let mut transfer_batches = AgeRange::default();
        let mut destroyed_batches = AgeRange::default();
        for chunk in write_chunks(range) {
            collect_age_range(
                transfer_volumes,
                &mut transfer_batches,
                chunk.start,
                chunk.end,
            );
            collect_age_range(
                source_coindays_destroyed,
                &mut destroyed_batches,
                chunk.start,
                chunk.end,
            );
            for offset in 0..chunk.len() {
                let volumes_btc = AgeRange::from_fn(|id| {
                    f64::from(Bitcoin::from(id.select(&transfer_batches)[offset]))
                });
                let destroyed =
                    AgeRange::from_fn(|id| f64::from(id.select(&destroyed_batches)[offset]));
                let consumed = allocate_consumed_coindays(volumes_btc, destroyed, &bounds);
                for (range, value) in self.ranges.iter_mut().zip(consumed.iter()) {
                    range.coindays_consumed.push_block(CoinDays::from(*value));
                }
            }
            write_locked(self.consumed_vecs_mut(), exit)?;
        }
        Ok(())
    }

    fn consumed_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.ranges
            .iter_mut()
            .flat_map(|range| range.coindays_consumed.iter_any_stored_mut())
    }

    fn compute_rest(
        &mut self,
        starting_height: Height,
        created: &AgeRange<&impl ReadableVec<Height, CoinDays>>,
        exit: &Exit,
    ) -> Result<()> {
        for id in AgeRangeId::ALL {
            let created = id.select(created);
            let RangeVecs {
                coindays_consumed,
                wakefulness_source: wakefulness,
                ..
            } = id.select_mut(&mut self.ranges);
            wakefulness.compute_transform2(
                starting_height,
                *created,
                &coindays_consumed.cumulative.height,
                |(height, created, consumed, ..)| {
                    debug_assert!(
                        consumed <= created,
                        "coin days consumed exceed those created"
                    );
                    (
                        height,
                        BoundedRatio::from(f64::from(consumed) / f64::from(created)),
                    )
                },
                exit,
            )?;
        }
        Ok(())
    }
}

fn age_bounds_days() -> AgeRange<(f64, f64)> {
    AgeRange::from_fn(|id| {
        let bound = id.bounds();
        let lower = bound.start as f64 / HOURS_PER_DAY;
        let width = if id == AgeRangeId::Over15Y {
            0.0
        } else {
            (bound.end - bound.start) as f64 / HOURS_PER_DAY
        };
        (lower, width)
    })
}

fn allocate_consumed_coindays(
    transfer_volume_btc: AgeRange<f64>,
    coindays_destroyed: AgeRange<f64>,
    bounds: &AgeRange<(f64, f64)>,
) -> AgeRange<f64> {
    let mut result = AgeRange::default();
    let mut older_transfer_volume = 0.0;

    for &id in AgeRangeId::ALL.iter().rev() {
        let (lower_days, width_days) = *id.select(bounds);
        let transfer_volume = *id.select(&transfer_volume_btc);
        let within_cohort =
            (*id.select(&coindays_destroyed) - transfer_volume * lower_days).max(0.0);
        *id.select_mut(&mut result) = within_cohort + older_transfer_volume * width_days;
        older_transfer_volume += transfer_volume;
    }

    result
}

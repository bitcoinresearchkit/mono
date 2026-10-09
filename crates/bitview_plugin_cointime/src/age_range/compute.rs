use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_compute::prepare_computed;
use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_primitives::{BoundedRatio, CoinDays};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Bitcoin, Height, Sats, Version};
use vecdb::{AnyStoredVec, AnyVec, CheckedSub, ReadableVec, WritableVec};

use super::{RangeVecs, Vecs};

const HOURS_PER_DAY: f64 = 24.0;
const WRITE_INTERVAL: usize = 10_000;

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, age: &AgeVecs, exit: &Exit) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let transfer_volumes = AgeRange::from_fn(|id| {
            &id.select(&age.cohorts.activity.transfer_volume.cohorts.age)
                .block
                .sats
        });
        let coindays_destroyed = AgeRange::from_fn(|id| {
            &id.select(&age.cohorts.activity.coindays_destroyed.cohorts.age)
                .block
        });
        let coindays_created =
            AgeRange::from_fn(|id| &id.select(&age.coindays_created).cumulative.height);

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
        let version = Version::combine_all(
            transfer_volumes
                .iter()
                .map(|vec| vec.version())
                .chain(source_coindays_destroyed.iter().map(|vec| vec.version())),
        );
        let source_end = transfer_volumes
            .iter()
            .map(|vec| vec.len())
            .chain(source_coindays_destroyed.iter().map(|vec| vec.len()))
            .min()
            .unwrap_or_default();
        let bounds = age_bounds_days();
        let start = prepare_computed(
            self.ranges
                .as_array_mut()
                .map(|range| range.coindays_consumed.stored_mut()),
            version,
            usize::from(starting_height).min(source_end),
            exit,
        )?;
        let mut chunk_start = start;
        while chunk_start < source_end {
            let chunk_end = (chunk_start + WRITE_INTERVAL).min(source_end);
            let transfer_batches = AgeRange::from_fn(|id| {
                id.select(transfer_volumes)
                    .collect_range_at(chunk_start, chunk_end)
            });
            let destroyed_batches = AgeRange::from_fn(|id| {
                id.select(source_coindays_destroyed)
                    .collect_range_at(chunk_start, chunk_end)
            });
            for offset in 0..chunk_end - chunk_start {
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
            let _lock = exit.lock();
            for range in self.ranges.iter_mut() {
                range.coindays_consumed.write()?;
            }
            chunk_start = chunk_end;
        }
        Ok(())
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
                coindays_stored: stored,
                wakefulness_source: activity,
                ..
            } = id.select_mut(&mut self.ranges);
            let consumed = &coindays_consumed.cumulative.height;
            let source_end = created.len().min(consumed.len());
            let mut start = prepare_computed(
                [stored.stored_mut(), activity],
                created.version() + consumed.version(),
                usize::from(starting_height).min(source_end),
                exit,
            )?;
            // Persist the common rewind before appending either output.
            let mut end = start;
            loop {
                let mut consumed = consumed.collect_range_at(start, end).into_iter();
                created.fold_range_at(start, end, (), |(), created| {
                    let consumed = consumed.next().unwrap();
                    stored.cumulative.height.push(
                        created
                            .checked_sub(consumed)
                            .expect("coindays stored underflow"),
                    );
                    activity.push(BoundedRatio::from(f64::from(consumed) / f64::from(created)));
                });
                let _lock = exit.lock();
                stored.write()?;
                activity.write()?;
                if end == source_end {
                    break;
                }
                start = end;
                end = stored
                    .cumulative
                    .height
                    .batch_end(source_end)
                    .min(activity.batch_end(source_end));
            }
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

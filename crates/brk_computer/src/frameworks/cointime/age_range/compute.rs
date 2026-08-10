use brk_cohort::{AgeRange, AgeRangeId};
use brk_error::Result;
use brk_indexer::Indexer;
use brk_types::{Bitcoin, Height, ONE_DAY_IN_SEC_F64, Sats, StoredF64, Timestamp, Version};
use vecdb::{AnyVec, CheckedSub, ColumnId, Exit, ReadableVec, StoredVec, WritableVec};

use super::Vecs;
use crate::{distribution, frameworks::WeightedCohortState, indexes};

const HOURS_PER_DAY: f64 = 24.0;
const WRITE_INTERVAL: usize = 10_000;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        distribution: &distribution::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution.utxo_cohorts.age_range)
                .metrics
                .supply
                .total
                .sats
                .height
        });
        let transfer_volumes = AgeRange::from_fn(|id| {
            &id.select(&distribution.utxo_cohorts.age_range)
                .metrics
                .activity
                .transfer_volume
                .block
                .sats
        });
        let coindays_destroyed = AgeRange::from_fn(|id| {
            &id.select(&distribution.utxo_cohorts.age_range)
                .metrics
                .activity
                .coindays_destroyed
                .block
        });

        self.compute_created(
            starting_height,
            &indexes.timestamp.monotonic,
            &supplies,
            exit,
        )?;
        self.compute_consumed(
            starting_height,
            &transfer_volumes,
            &coindays_destroyed,
            exit,
        )?;
        self.compute_rest(starting_height, &supplies, exit)
    }

    fn compute_created<T, S>(
        &mut self,
        starting_height: Height,
        timestamps: &T,
        supplies: &AgeRange<&S>,
        exit: &Exit,
    ) -> Result<()>
    where
        T: ReadableVec<Height, Timestamp>,
        S: ReadableVec<Height, Sats>,
    {
        let version: Version = std::iter::once(timestamps.version())
            .chain(supplies.iter().map(|vec| vec.version()))
            .sum();
        let source_end = supplies
            .iter()
            .map(|vec| vec.len())
            .chain(std::iter::once(timestamps.len()))
            .min()
            .unwrap_or_default();
        self.coindays_created.cumulative.compute_batched_to(
            starting_height,
            source_end,
            version,
            WRITE_INTERVAL,
            |created, range| {
                let mut cumulative = created
                    .collect_last()
                    .unwrap_or_else(|| AgeRangeId::from_fn(|_| StoredF64::default()));
                let timestamp_start = range.start.saturating_sub(1);
                let timestamp_batch = timestamps.collect_range_at(timestamp_start, range.end);
                let supply_batches = AgeRange::from_fn(|id| {
                    id.select(supplies).collect_range_at(range.start, range.end)
                });

                for offset in 0..range.len() {
                    let interval_seconds =
                        monotonic_interval_seconds(&timestamp_batch, range.start, offset);
                    created.push(AgeRangeId::from_fn(|column| {
                        let value = column.get_mut(&mut cumulative);
                        *value += coindays_created(
                            column.select(&supply_batches)[offset],
                            interval_seconds,
                        );
                        *value
                    }));
                }

                Ok(())
            },
            exit,
        )?;
        Ok(())
    }

    fn compute_consumed<V, D>(
        &mut self,
        starting_height: Height,
        transfer_volumes: &AgeRange<&V>,
        source_coindays_destroyed: &AgeRange<&D>,
        exit: &Exit,
    ) -> Result<()>
    where
        V: ReadableVec<Height, Sats>,
        D: ReadableVec<Height, StoredF64>,
    {
        let version: Version = transfer_volumes
            .iter()
            .map(|vec| vec.version())
            .chain(source_coindays_destroyed.iter().map(|vec| vec.version()))
            .sum();
        let source_end = transfer_volumes
            .iter()
            .map(|vec| vec.len())
            .chain(source_coindays_destroyed.iter().map(|vec| vec.len()))
            .min()
            .unwrap_or_default();
        let bounds = age_bounds_days();
        self.coindays_consumed.cumulative.compute_batched_to(
            starting_height,
            source_end,
            version,
            WRITE_INTERVAL,
            |target, range| {
                let mut cumulative = target
                    .collect_last()
                    .unwrap_or_else(|| AgeRangeId::from_fn(|_| StoredF64::default()));
                let transfer_batches = AgeRange::from_fn(|id| {
                    id.select(transfer_volumes)
                        .collect_range_at(range.start, range.end)
                });
                let destroyed_batches = AgeRange::from_fn(|id| {
                    id.select(source_coindays_destroyed)
                        .collect_range_at(range.start, range.end)
                });

                for offset in 0..range.len() {
                    let volumes_btc = AgeRange::from_fn(|id| {
                        f64::from(Bitcoin::from(id.select(&transfer_batches)[offset]))
                    });
                    let coindays_destroyed =
                        AgeRange::from_fn(|id| f64::from(id.select(&destroyed_batches)[offset]));
                    let consumed =
                        allocate_consumed_coindays(volumes_btc, coindays_destroyed, &bounds);
                    target.push(AgeRangeId::from_fn(|column| {
                        let value = column.get_mut(&mut cumulative);
                        *value += StoredF64::from(*column.select(&consumed));
                        *value
                    }));
                }

                Ok(())
            },
            exit,
        )?;
        Ok(())
    }

    fn compute_rest<S>(
        &mut self,
        starting_height: Height,
        supplies: &AgeRange<&S>,
        exit: &Exit,
    ) -> Result<()>
    where
        S: ReadableVec<Height, Sats>,
    {
        {
            let created = self.coindays_created.cumulative.read_only_clone();
            let consumed = self.coindays_consumed.cumulative.read_only_clone();
            self.coindays_stored.cumulative.compute_transform2_batched(
                starting_height,
                &created,
                &consumed,
                WRITE_INTERVAL,
                |(height, created, consumed, ..)| {
                    let stored = AgeRangeId::from_fn(|column| {
                        (*column.get(&created))
                            .checked_sub(*column.get(&consumed))
                            .expect("coindays stored underflow")
                    });
                    (height, stored)
                },
                exit,
            )?;
            self.activity.height.compute_transform2_batched(
                starting_height,
                &consumed,
                &created,
                WRITE_INTERVAL,
                |(height, consumed, created, ..)| {
                    let wakefulness = AgeRangeId::from_fn(|column| {
                        *column.get(&consumed) / *column.get(&created)
                    });
                    (height, wakefulness)
                },
                exit,
            )?;
        }

        self.compute_supply(starting_height, supplies, exit)
    }

    fn compute_supply<S>(
        &mut self,
        starting_height: Height,
        supplies: &AgeRange<&S>,
        exit: &Exit,
    ) -> Result<()>
    where
        S: ReadableVec<Height, Sats>,
    {
        let wakefulness = self.activity.height.read_only_clone();
        let source_version: Version = std::iter::once(wakefulness.version())
            .chain(supplies.iter().map(|vec| vec.version()))
            .sum();
        let supply = &mut self.supply;
        supply
            .awake
            .validate_computed_version_or_reset(source_version)?;
        supply
            .dormant
            .validate_computed_version_or_reset(source_version)?;

        let start = [supply.awake.height.len(), supply.dormant.height.len()]
            .into_iter()
            .min()
            .unwrap_or_default()
            .min(usize::from(starting_height));
        supply.awake.truncate_if_needed_at(start)?;
        supply.dormant.truncate_if_needed_at(start)?;

        let source_end = supplies
            .iter()
            .map(|vec| vec.len())
            .chain(std::iter::once(wakefulness.len()))
            .min()
            .unwrap_or_default();
        let mut chunk_start = start;
        while chunk_start < source_end {
            let chunk_end = (chunk_start + WRITE_INTERVAL).min(source_end);
            let supply_batches = AgeRange::from_fn(|id| {
                id.select(supplies).collect_range_at(chunk_start, chunk_end)
            });
            let wakefulness_batch = wakefulness.collect_range_at(chunk_start, chunk_end);

            for (offset, weights) in wakefulness_batch.iter().enumerate() {
                let split = AgeRange::from_fn(|id| {
                    WeightedCohortState::split_supply(
                        id.select(&supply_batches)[offset],
                        *id.get(weights),
                    )
                });
                supply
                    .awake
                    .push(AgeRangeId::from_fn(|id| id.select(&split).0));
                supply
                    .dormant
                    .push(AgeRangeId::from_fn(|id| id.select(&split).1));
            }

            {
                let _lock = exit.lock();
                supply.awake.write()?;
                supply.dormant.write()?;
            }
            chunk_start = chunk_end;
        }

        Ok(())
    }
}

#[inline(always)]
fn monotonic_interval_seconds(
    timestamp_batch: &[Timestamp],
    chunk_start: usize,
    offset: usize,
) -> u32 {
    if chunk_start + offset == 0 {
        return 0;
    }

    let current_index = offset + usize::from(chunk_start > 0);
    (*timestamp_batch[current_index]).saturating_sub(*timestamp_batch[current_index - 1])
}

#[inline(always)]
fn coindays_created(supply: Sats, interval_seconds: u32) -> StoredF64 {
    StoredF64::from(f64::from(Bitcoin::from(supply)) * interval_seconds as f64 / ONE_DAY_IN_SEC_F64)
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

#[cfg(test)]
mod tests {
    use brk_cohort::AGE_RANGE_COUNT;

    use super::*;

    #[test]
    fn created_coindays_use_the_exact_monotonic_block_interval() {
        let created = f64::from(coindays_created(Sats::ONE_BTC, 6 * 60 * 60));

        assert!((created - 0.25).abs() < 1e-12);
    }

    #[test]
    fn monotonic_interval_handles_initial_and_resumed_chunks() {
        let initial = [
            Timestamp::from(100_u32),
            Timestamp::from(160_u32),
            Timestamp::from(220_u32),
        ];
        let resumed = [Timestamp::from(160_u32), Timestamp::from(220_u32)];

        assert_eq!(monotonic_interval_seconds(&initial, 0, 0), 0);
        assert_eq!(monotonic_interval_seconds(&initial, 0, 1), 60);
        assert_eq!(monotonic_interval_seconds(&resumed, 2, 0), 60);
    }

    #[test]
    fn destruction_at_a_boundary_stays_in_the_ranges_already_traversed() {
        let mut volumes = AgeRange::default();
        let mut cdd = AgeRange::default();
        volumes._1d_to_1w = 1.0;
        cdd._1d_to_1w = 1.0;

        let allocated = allocate_consumed_coindays(volumes, cdd, &age_bounds_days());

        assert!((allocated.under_1h - 1.0 / HOURS_PER_DAY).abs() < 1e-12);
        assert!((allocated._1h_to_1d - 23.0 / HOURS_PER_DAY).abs() < 1e-12);
        assert!(allocated._1d_to_1w.abs() < 1e-12);
        assert!((allocated.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn consumed_coindays_cover_every_traversed_cohort() {
        let mut volumes = AgeRange::default();
        let mut cdd = AgeRange::default();
        volumes._1d_to_1w = 2.0;
        cdd._1d_to_1w = 20.0;

        let allocated = allocate_consumed_coindays(volumes, cdd, &age_bounds_days());

        assert!((allocated.under_1h - 2.0 / HOURS_PER_DAY).abs() < 1e-12);
        assert!((allocated._1h_to_1d - 46.0 / HOURS_PER_DAY).abs() < 1e-12);
        assert!((allocated._1d_to_1w - 18.0).abs() < 1e-12);
        assert!((allocated.iter().sum::<f64>() - 20.0).abs() < 1e-12);
    }

    #[test]
    fn allocated_coindays_conserve_mixed_cohort_destruction() {
        let bounds = age_bounds_days();
        let mut volumes = AgeRange::default();
        let mut cdd = AgeRange::default();

        for id in [
            AgeRangeId::Under1H,
            AgeRangeId::From1HTo1D,
            AgeRangeId::From1DTo1W,
            AgeRangeId::From9MTo1Y,
            AgeRangeId::From10YTo12Y,
            AgeRangeId::From12YTo15Y,
            AgeRangeId::Over15Y,
        ] {
            let (lower, width) = *id.select(&bounds);
            let volume = id.index() as f64 + 1.0;
            let age = lower + if width > 0.0 { width / 2.0 } else { 30.0 };
            *id.select_mut(&mut volumes) = volume;
            *id.select_mut(&mut cdd) = volume * age;
        }

        let expected = cdd.iter().sum::<f64>();
        let allocated = allocate_consumed_coindays(volumes, cdd, &bounds);

        assert!((allocated.iter().sum::<f64>() - expected).abs() < 1e-9);
    }

    #[test]
    fn bounds_and_allocation_cover_every_canonical_age_range() {
        let bounds = age_bounds_days();
        let mut volumes = AgeRange::default();
        let mut cdd = AgeRange::default();

        volumes.over_15y = 1.0;
        cdd.over_15y = bounds.over_15y.0 + 30.0;

        let expected = cdd.iter().sum::<f64>();
        let allocated = allocate_consumed_coindays(volumes, cdd, &bounds);

        assert_eq!(bounds.iter().count(), AGE_RANGE_COUNT);
        assert!(allocated.over_15y > 0.0);
        assert!((allocated.iter().sum::<f64>() - expected).abs() < 1e-9);
    }
}

use std::iter;

use bitview_cohort::AgeRange;
use bitview_compute::{AgeBand, prepare_computed};
use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Timestamp, Version};
use vecdb::{AnyStoredVec, AnyVec, ReadableVec};

use super::Vecs;
use crate::{Dependencies, model::PrimaryBatch};

const WRITE_INTERVAL: usize = 20_000;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;
        self.compute_primary(dependencies, context.exit())?;
        let supplies = dependencies
            .distribution_age
            .cohorts
            .supply
            .total
            .age_supplies();
        let weights =
            AgeRange::from_fn(|id| &id.select(&self.age_range.spending_exposure.mobility).height);
        self.urpd.compute(
            dependencies.distribution_age.cohorts.all_supply().version()
                + dependencies.urpd.timestamps.version(),
            usize::from(dependencies.indexer.safe_lengths().height),
            dependencies.urpd,
            &weights,
            &supplies,
            context.exit(),
        )?;
        context.compact_database(&self.db);
        Ok(())
    }
}

impl Vecs {
    fn compute_primary(&mut self, dependencies: Dependencies<'_>, exit: &Exit) -> Result<()> {
        let Dependencies {
            indexer,
            urpd: _,
            mappings,
            distribution_age,
        } = dependencies;
        let starting_lengths = indexer.safe_lengths();
        let transfer_volumes = AgeRange::from_fn(|id| {
            &id.select(
                &distribution_age
                    .cohorts
                    .activity
                    .transfer_volume
                    .cohorts
                    .age,
            )
            .cumulative
            .sats
            .height
        });
        let supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution_age.cohorts.supply.total.cohorts.age)
                .sats
                .height
        });
        let loss_supplies = AgeRange::from_fn(|id| {
            &id.select(&distribution_age.cohorts.supply.in_loss.cohorts.age)
                .sats
                .height
        });
        let realized_caps = AgeRange::from_fn(|id| {
            &id.select(&distribution_age.cohorts.realized.cap.cohorts.age)
                .cents
                .height
        });
        let cap_raw =
            AgeRange::from_fn(|id| id.select(&distribution_age.cohorts.realized.cap_raw.age));
        let capitalized_cap_raw = AgeRange::from_fn(|id| {
            id.select(&distribution_age.cohorts.realized.capitalized_cap_raw.age)
        });
        let coindays_created = AgeRange::from_fn(|id| {
            &id.select(&distribution_age.coindays_created)
                .cumulative
                .height
        });

        let timestamps = &mappings.timestamp.monotonic;

        let source_version = Version::combine_all(
            iter::once(timestamps.version())
                .chain(transfer_volumes.iter().map(|vec| vec.version()))
                .chain(coindays_created.iter().map(|vec| vec.version()))
                .chain(supplies.iter().map(|vec| vec.version()))
                .chain(loss_supplies.iter().map(|vec| vec.version()))
                .chain(realized_caps.iter().map(|vec| vec.version()))
                .chain(cap_raw.iter().map(|vec| vec.version()))
                .chain(capitalized_cap_raw.iter().map(|vec| vec.version())),
        );

        let start = prepare_computed(
            self.primary_vecs_mut().collect::<Vec<_>>(),
            source_version,
            usize::from(starting_lengths.height),
        )?;

        let source_end = transfer_volumes
            .iter()
            .map(|vec| vec.len())
            .chain(coindays_created.iter().map(|vec| vec.len()))
            .chain(supplies.iter().map(|vec| vec.len()))
            .chain(loss_supplies.iter().map(|vec| vec.len()))
            .chain(realized_caps.iter().map(|vec| vec.len()))
            .chain(cap_raw.iter().map(|vec| vec.len()))
            .chain(capitalized_cap_raw.iter().map(|vec| vec.len()))
            .chain(iter::once(timestamps.len()))
            .min()
            .unwrap_or_default();

        if source_end == 0 {
            return Ok(());
        }

        let genesis_timestamp = timestamps
            .collect_one(Height::ZERO)
            .unwrap_or(Timestamp::ZERO);
        let bounds = AgeBand::all();
        let mut chunk_start = start;
        while chunk_start < source_end {
            let chunk_end = (chunk_start + WRITE_INTERVAL).min(source_end);
            let batch = PrimaryBatch::collect(
                timestamps,
                &transfer_volumes,
                &coindays_created,
                &supplies,
                &loss_supplies,
                &realized_caps,
                &cap_raw,
                &capitalized_cap_raw,
                chunk_start,
                chunk_end,
            );
            for values in batch.primary_values_batch(genesis_timestamp, &bounds) {
                self.age_range.push(
                    &values.spending_rate,
                    &values.spending_exposure,
                    &values.mobility,
                );
                self.aggregate_sources.push(values);
            }

            {
                let _lock = exit.lock();
                for vec in self.primary_vecs_mut() {
                    vec.write()?;
                }
            }
            chunk_start = chunk_end;
        }

        Ok(())
    }

    fn primary_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.age_range
            .stored_vecs_mut()
            .chain(self.aggregate_sources.stored_vecs_mut())
    }
}

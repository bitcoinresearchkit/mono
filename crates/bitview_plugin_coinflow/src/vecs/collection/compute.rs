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
        let accounting = distribution_age.accounting_sources();
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
                .chain(iter::once(accounting.version())),
        );

        let source_end = transfer_volumes
            .iter()
            .map(|vec| vec.len())
            .chain(coindays_created.iter().map(|vec| vec.len()))
            .chain(iter::once(accounting.len()))
            .chain(iter::once(timestamps.len()))
            .min()
            .unwrap_or_default();

        let start = prepare_computed(
            self.primary_vecs_mut().collect::<Vec<_>>(),
            source_version,
            usize::from(starting_lengths.height).min(source_end),
            exit,
        )?;

        if source_end == 0 {
            return Ok(());
        }

        let genesis_timestamp = timestamps
            .collect_one(Height::ZERO)
            .unwrap_or(Timestamp::ZERO);
        let bounds = AgeBand::all();
        let mut batch = PrimaryBatch::default();
        let mut chunk_start = start;
        while chunk_start < source_end {
            let chunk_end = (chunk_start + WRITE_INTERVAL).min(source_end);
            batch.collect_into(
                timestamps,
                &transfer_volumes,
                &coindays_created,
                &accounting,
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

use std::iter;

use bitview_cohort::AgeRange;
use bitview_compute::{AgeBand, prepare_computed};
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_urpd::compute_cost_basis;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Timestamp, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableVec};

use crate::{Dependencies, Vecs, model::PrimaryBatch};

const WRITE_INTERVAL: usize = 20_000;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        self.compute_primary(dependencies, context.exit())?;
        let supplies = dependencies.age.age_supplies();
        let weights = self.age_ranges.urpd_weight_sources();
        compute_cost_basis(
            &mut self.urpd_replay,
            self.aggregate
                .cohorts
                .as_array_mut()
                .map(|cohort| &mut cohort.mobile.cost_basis),
            dependencies.age.all_supply().version() + dependencies.urpd.timestamps.version(),
            usize::from(dependencies.indexer.safe_lengths().height),
            dependencies.urpd,
            &weights,
            &supplies,
            context.exit(),
        )?;
        Ok(())
    }
}

impl Vecs {
    fn compute_primary(&mut self, dependencies: Dependencies<'_>, exit: &Exit) -> Result<()> {
        let Dependencies {
            indexer,
            urpd: _,
            mappings,
            age,
        } = dependencies;
        let starting_lengths = indexer.safe_lengths();
        let transfer_volumes = AgeRange::from_fn(|id| {
            &id.select(&age.ranges)
                .activity
                .transfer_volume
                .value
                .cumulative
                .sats
                .height
        });
        let accounting = age.accounting_sources();
        let coindays_created =
            AgeRange::from_fn(|id| &id.select(&age.ranges).coindays_created.cumulative.height);

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
            .chain(iter::once(accounting.min_len()))
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
                self.age_ranges.push(&values);
                self.aggregate.sources.push(&values.cohorts);
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
        self.age_ranges
            .stored_vecs_mut()
            .chain(self.aggregate.sources.vecs_mut())
    }
}

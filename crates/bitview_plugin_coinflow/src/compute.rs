use std::iter;

use bitview_cohort::AgeRange;
use bitview_compute::{Inputs, write_chunks, write_locked};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_primitives::Lengths;
use bitview_traversable::Traversable;
use bitview_urpd::compute_cost_basis;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use vecdb::{AnyStoredVec, AnyVec, ReadableVec};

use crate::{
    Dependencies, Vecs,
    model::{AgeBand, PrimaryBatch},
};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let starting_lengths = context.start();
        self.compute_primary(dependencies, starting_lengths, context.exit())?;
        let supplies = dependencies.age.age_supplies();
        let weights = self.age_ranges.urpd_weight_sources();
        compute_cost_basis(
            &mut self.urpd_replay,
            self.aggregate
                .cohorts
                .as_array_mut()
                .map(|cohort| &mut cohort.mobile.cost_basis),
            dependencies.age.all_supply().version() + dependencies.urpd.timestamps.version(),
            usize::from(starting_lengths.height),
            dependencies.urpd,
            &weights,
            &supplies,
            &dependencies.price.spot.cents.height,
            context.exit(),
        )
    }
}

impl Vecs {
    fn compute_primary(
        &mut self,
        dependencies: Dependencies<'_>,
        starting_lengths: Lengths,
        exit: &Exit,
    ) -> Result<()> {
        let Dependencies {
            urpd: _,
            mappings,
            price: _,
            age,
        } = dependencies;
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
        let mut inputs = Inputs::new(
            accounting.version(),
            iter::once(timestamps as &dyn AnyVec)
                .chain(transfer_volumes.iter().map(|vec| *vec as &dyn AnyVec))
                .chain(coindays_created.iter().map(|vec| *vec as &dyn AnyVec)),
        );
        inputs.len = inputs.len.min(accounting.min_len());
        let range = inputs.prepare(
            self.primary_vecs_mut().collect::<Vec<_>>(),
            usize::from(starting_lengths.height),
            exit,
        )?;
        if range.end == 0 {
            return Ok(());
        }

        let genesis_timestamp = timestamps
            .collect_one(Height::ZERO)
            .expect("timestamps cover the nonempty range");
        let bounds = AgeBand::all();
        let mut batch = PrimaryBatch::default();
        for chunk in write_chunks(range) {
            batch.collect_into(
                timestamps,
                &transfer_volumes,
                &coindays_created,
                &accounting,
                chunk.start,
                chunk.end,
            );
            for values in batch.primary_values_batch(genesis_timestamp, &bounds) {
                self.age_ranges.push(&values);
                self.aggregate.sources.push(&values.cohorts);
            }
            write_locked(self.primary_vecs_mut(), exit)?;
        }

        Ok(())
    }

    fn primary_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.age_ranges
            .iter_any_stored_mut()
            .chain(self.aggregate.sources.iter_any_stored_mut())
    }
}

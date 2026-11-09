use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::Result;
use rayon::{join, prelude::*};

use crate::{Dependencies, Vecs, component::Component, inner::RarityMeterInner};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let starting_lengths = context.start();
        let exit = context.exit();
        let Dependencies {
            holders,
            cointime,
            coinflow,
            spot_cents,
        } = dependencies;

        let (components_result, extremes_result) = join(
            || {
                self.components.compute(
                    holders,
                    cointime,
                    coinflow,
                    spot_cents,
                    starting_lengths,
                    exit,
                )
            },
            || self.extremes.compute(starting_lengths.height, exit),
        );
        components_result?;
        extremes_result?;

        let starting_height = starting_lengths.height;
        let local = self.components.local();
        let cycle = self.components.cycle();
        // Full combines all 16 components itself: with disputed ranges reading as neither, its
        // boundaries do not follow from Local's and Cycle's.
        let full: Vec<_> = local.iter().chain(&cycle).copied().collect();
        let jobs: [(&mut RarityMeterInner, &[&Component]); 3] = [
            (&mut self.local, &local),
            (&mut self.cycle, &cycle),
            (&mut self.full, &full),
        ];
        let has_work = jobs.iter().any(|(inner, components)| {
            inner.needs_compute(components, spot_cents, starting_height)
        });
        let compute = |(inner, components): (&mut RarityMeterInner, &[&Component])| {
            inner.compute(components, spot_cents, starting_height, exit)
        };
        if has_work {
            jobs.into_par_iter().try_for_each(compute)
        } else {
            jobs.into_iter().try_for_each(compute)
        }
    }
}

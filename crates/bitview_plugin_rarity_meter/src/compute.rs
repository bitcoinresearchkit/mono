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
        let exit = context.exit();
        let Dependencies {
            indexer,
            holders,
            cointime,
            coinflow,
            price: prices,
        } = dependencies;

        let spot = &prices.spot.cents.height;
        let metrics = &holders.cohorts.all;
        let realized = &metrics.realized;
        let (components_result, extremes_result) = join(
            || {
                self.components
                    .compute(indexer, holders, cointime, coinflow, spot, exit)
            },
            || {
                self.extremes.compute(
                    indexer,
                    &metrics.supply.in_loss.block.btc.height,
                    &realized.profit.sum._24h.usd.height,
                    &realized.loss.sum._24h.usd.height,
                    &realized.peak_regret.sum._24h.usd.height,
                    &realized.sell_side_risk_ratio._24h.ratio.height,
                    exit,
                )
            },
        );
        components_result?;
        extremes_result?;

        let starting_height = indexer.compute_start().height;
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
        let has_work = jobs
            .iter()
            .any(|(inner, components)| inner.needs_compute(components, spot, starting_height));
        let compute = |(inner, components): (&mut RarityMeterInner, &[&Component])| {
            inner.compute(components, spot, starting_height, exit)
        };
        if has_work {
            jobs.into_par_iter().try_for_each(compute)
        } else {
            jobs.into_iter().try_for_each(compute)
        }
    }
}

use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_types::{Cents, Height};
use rayon::{join, prelude::*};
use vecdb::{Database, LazyVec};

use crate::{Dependencies, Vecs, component::Component, inner::RarityMeterInner};

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
        let exit = context.exit();
        let Dependencies {
            indexer,
            bedrock,
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
                    &metrics.supply.in_loss.btc.height,
                    &realized.profit.sum._24h.usd.height,
                    &realized.loss.sum._24h.usd.height,
                    &realized.peak_regret.sum._24h.usd.height,
                    &metrics.ratios.sell_side_risk_ratio._24h.percent.height,
                    exit,
                )
            },
        );
        components_result?;
        extremes_result?;

        let local_components = [
            &self.components.under_4m_realized_price,
            &self.components.under_6m_realized_price,
            &self.components.sth_realized_price,
            &self.components.sth_capitalized_price,
        ];

        // Bedrock floors run from the rarest low boundary to the broadest one,
        // matching the rarity meter's P0.1, P0.5, P1, P2, and P5 order.
        let bedrock_floors = [
            [
                &bedrock.raw.floor.pct99_9.cents.height,
                &bedrock.raw.floor.pct99_5.cents.height,
                &bedrock.raw.floor.pct99.cents.height,
                &bedrock.raw.floor.pct98.cents.height,
                &bedrock.raw.floor.pct95.cents.height,
            ],
            [
                &bedrock.cointime.floor.pct99_9.cents.height,
                &bedrock.cointime.floor.pct99_5.cents.height,
                &bedrock.cointime.floor.pct99.cents.height,
                &bedrock.cointime.floor.pct98.cents.height,
                &bedrock.cointime.floor.pct95.cents.height,
            ],
            [
                &bedrock.coinflow.floor.pct99_9.cents.height,
                &bedrock.coinflow.floor.pct99_5.cents.height,
                &bedrock.coinflow.floor.pct99.cents.height,
                &bedrock.coinflow.floor.pct98.cents.height,
                &bedrock.coinflow.floor.pct95.cents.height,
            ],
        ];

        let cycle_components = [
            &self.components.over_4m_realized_price,
            &self.components.over_6m_realized_price,
            &self.components.realized_price,
            &self.components.capitalized_price,
            &self.components.lth_realized_price,
            &self.components.lth_capitalized_price,
        ];
        let local_v2_components = [
            &self.components.under_4m_realized_price,
            &self.components.under_6m_realized_price,
            &self.components.under_4m_capitalized_price,
            &self.components.under_6m_capitalized_price,
            &self.components.sth_realized_price,
            &self.components.sth_capitalized_price,
            &self.components.sth_median_price_btc_weighted.component,
            &self.components.sth_median_price_usd_weighted.component,
        ];
        let cycle_v2_components = [
            &self.components.realized_price,
            &self.components.capitalized_price,
            &self.components.median_price_btc_weighted.component,
            &self.components.median_price_usd_weighted.component,
            &self.components.cointime_median_price_btc_weighted.component,
            &self.components.cointime_median_price_usd_weighted.component,
            &self.components.coinflow_median_price_btc_weighted.component,
            &self.components.coinflow_median_price_usd_weighted.component,
            &self.components.over_6m_realized_price,
            &self.components.over_4m_realized_price,
            &self.components.vaulted_price,
            &self.components.active_price,
            &self.components.true_market_mean_price,
            &self.components.cointime_price,
            &self.components.awake_price,
            &self.components.coinflow_price,
        ];
        let starting_height = indexer.safe_lengths().height;
        let jobs: [(
            &mut RarityMeterInner,
            &[&Component],
            &[[&LazyVec<Height, Cents, Height, Cents>; 5]],
            Height,
        ); 4] = [
            (&mut self.local, &local_components, &[], starting_height),
            (
                &mut self.cycle,
                &cycle_components,
                &bedrock_floors,
                starting_height,
            ),
            (
                &mut self.local_v2,
                &local_v2_components,
                &[],
                starting_height,
            ),
            (
                &mut self.cycle_v2,
                &cycle_v2_components,
                &bedrock_floors,
                starting_height,
            ),
        ];
        let has_work = jobs
            .iter()
            .any(|(inner, components, lower_components, start)| {
                inner.needs_compute(components, lower_components, spot, *start)
            });
        let compute = |(inner, components, lower_components, start)| {
            RarityMeterInner::compute(inner, components, lower_components, spot, start, exit)
        };

        if has_work {
            jobs.into_par_iter().try_for_each(compute)?;
        } else {
            jobs.into_iter().try_for_each(compute)?;
        }

        // Full inherits every boundary and score from Local and Cycle.
        self.full
            .compute_combined(&[&self.local, &self.cycle], spot, starting_height, exit)?;

        self.full_v2.compute_combined(
            &[&self.local_v2, &self.cycle_v2],
            spot,
            starting_height,
            exit,
        )?;

        Ok(())
    }
}

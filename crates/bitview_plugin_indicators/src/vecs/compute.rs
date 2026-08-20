use brk_error::Result;

use bitview_compute::RatioDollars;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Indexer;
use brk_types::{Dollars, PartsPerMillion64, StoredF32};
use vecdb::Exit;

use super::Vecs;
use crate::{Dependencies, gini};

impl Vecs {
    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        mining: &bitview_plugin_mining::Vecs,
        distribution: &bitview_plugin_distribution::Vecs,
        market: &bitview_plugin_market::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let starting_lengths = indexer.safe_lengths();

        // Puell Multiple: daily_subsidy_usd / sma_365d_subsidy_usd
        self.puell_multiple
            .ppm
            .compute_binary::<Dollars, Dollars, RatioDollars<PartsPerMillion64>>(
                starting_lengths.height,
                &mining.rewards.subsidy.block.usd,
                &mining.rewards.subsidy.average._1y.usd.height,
                exit,
            )?;

        // Gini coefficient (UTXO distribution inequality)
        gini::compute(&mut self.gini, distribution, indexer, exit)?;

        // RHODL Ratio: 1d-1w realized cap / 1y-2y realized cap
        self.rhodl_ratio.ppm.height.compute_transform3(
            starting_lengths.height,
            &distribution
                .cohorts
                .realized
                .cap
                .cohorts
                .age
                .range
                ._1d_to_1w
                .usd
                .height,
            &distribution
                .cohorts
                .realized
                .cap
                .cohorts
                .age
                .range
                ._1y_to_18m
                .usd
                .height,
            &distribution
                .cohorts
                .realized
                .cap
                .cohorts
                .age
                .range
                ._18m_to_2y
                .usd
                .height,
            |(i, young_cap, year1_cap, month18_cap, ..)| {
                let denominator = year1_cap + month18_cap;
                let ratio = f64::from(young_cap) / f64::from(denominator);
                (
                    i,
                    if ratio.is_finite() {
                        PartsPerMillion64::from(ratio)
                    } else {
                        PartsPerMillion64::default()
                    },
                )
            },
            exit,
        )?;

        let supply = &distribution.cohorts.supply;
        let supply_total_sats = &supply.total.cohorts.all.sats.height;

        // Seller Exhaustion Constant: % supply_in_profit × 30d_volatility
        self.seller_exhaustion.height.compute_transform3(
            starting_lengths.height,
            &supply.in_profit.cohorts.all.sats.height,
            &market.volatility._1m.height,
            supply_total_sats,
            |(i, profit_sats, volatility, total_sats, ..)| {
                let total = total_sats.as_u128() as f64;
                if total == 0.0 {
                    (i, StoredF32::from(0.0f32))
                } else {
                    let pct_in_profit = profit_sats.as_u128() as f64 / total;
                    (
                        i,
                        StoredF32::from((pct_in_profit * f64::from(volatility)) as f32),
                    )
                }
            },
            exit,
        )?;

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        self.compute_inner(
            dependencies.indexer,
            dependencies.mining,
            dependencies.distribution,
            dependencies.market,
            context.exit(),
        )
    }
}

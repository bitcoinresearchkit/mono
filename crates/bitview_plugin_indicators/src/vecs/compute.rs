use brk_error::Result;

use bitview_compute::RatioDollars;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Indexer;
use brk_types::{Dollars, PartsPerMillion64, StoredF32};
use rayon::join;
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

        let starting_height = indexer.safe_lengths().height;
        let Self {
            puell_multiple,
            gini,
            rhodl_ratio,
            seller_exhaustion,
            ..
        } = self;
        let subsidy = &mining.rewards.subsidy;
        let realized_cap = &distribution.cohorts.realized.cap.cohorts.age.range;
        let supply = &distribution.cohorts.supply;
        let supply_total_sats = &supply.total.cohorts.all.sats.height;

        let compute_puell = || {
            puell_multiple
                .ppm
                .compute_binary::<Dollars, Dollars, RatioDollars<PartsPerMillion64>>(
                    starting_height,
                    &subsidy.block.usd,
                    &subsidy.average._1y.usd.height,
                    exit,
                )
        };
        let compute_gini = || gini::compute(gini, distribution, starting_height, exit);
        let compute_rhodl = || {
            rhodl_ratio.ppm.height.compute_transform3(
                starting_height,
                &realized_cap._1d_to_1w.usd.height,
                &realized_cap._1y_to_18m.usd.height,
                &realized_cap._18m_to_2y.usd.height,
                |(height, young_cap, year1_cap, month18_cap, ..)| {
                    let denominator = year1_cap + month18_cap;
                    let ratio = f64::from(young_cap) / f64::from(denominator);
                    (
                        height,
                        if ratio.is_finite() {
                            PartsPerMillion64::from(ratio)
                        } else {
                            PartsPerMillion64::default()
                        },
                    )
                },
                exit,
            )
        };
        let compute_seller_exhaustion = || {
            seller_exhaustion.height.compute_transform3(
                starting_height,
                &supply.in_profit.cohorts.all.sats.height,
                &market.volatility._1m.height,
                supply_total_sats,
                |(height, profit_sats, volatility, total_sats, ..)| {
                    let total = total_sats.as_u128() as f64;
                    if total == 0.0 {
                        (height, StoredF32::from(0.0f32))
                    } else {
                        let pct_in_profit = profit_sats.as_u128() as f64 / total;
                        (
                            height,
                            StoredF32::from((pct_in_profit * f64::from(volatility)) as f32),
                        )
                    }
                },
                exit,
            )
        };

        let ((puell_result, gini_result), (rhodl_result, seller_exhaustion_result)) = join(
            move || join(compute_puell, compute_gini),
            move || join(compute_rhodl, compute_seller_exhaustion),
        );
        puell_result?;
        gini_result?;
        rhodl_result?;
        seller_exhaustion_result?;

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

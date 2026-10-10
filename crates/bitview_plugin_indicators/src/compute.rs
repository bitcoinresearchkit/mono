use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_primitives::{BasisPoints32, PartsPerMillion64, Ratio};
use bitview_transforms::RatioDollars;
use brk_error::Result;
use rayon::join;
use vecdb::{BinaryTransform, Database};

use crate::{Dependencies, Vecs, gini};

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
        let Dependencies {
            indexer,
            mining,
            age,
            holders,
            utxos,
            market,
        } = dependencies;
        let exit = context.exit();

        let starting_height = indexer.safe_lengths().height;
        let Self {
            puell_multiple,
            gini,
            rhodl_ratio,
            seller_exhaustion_constant,
            ..
        } = self;
        let subsidy = &mining.rewards.subsidy;
        let ranges = &age.ranges;
        let supply = &holders.cohorts.all.supply;
        let supply_total_sats = &supply.total.sats.height;

        // Daily issuance over its 365-day daily mean: the 24-hour sum over the 365-day sum / 365.
        let compute_puell = || {
            puell_multiple.fixed.height.compute_transform2(
                starting_height,
                &subsidy.sum._24h.usd.height,
                &subsidy.sum._1y.usd.height,
                |(height, day, year, ..)| {
                    (
                        height,
                        RatioDollars::<BasisPoints32>::apply(day * 365.0, year),
                    )
                },
                exit,
            )
        };
        let compute_gini = || gini::compute(gini, utxos, starting_height, exit);
        let compute_rhodl = || {
            rhodl_ratio.fixed.height.compute_transform3(
                starting_height,
                &ranges._1d_to_1w.realized.cap.value.usd.height,
                &ranges._1y_to_18m.realized.cap.value.usd.height,
                &ranges._18m_to_2y.realized.cap.value.usd.height,
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
            seller_exhaustion_constant.height.compute_transform3(
                starting_height,
                &supply.in_profit.sats.height,
                &market.volatility._1m.height,
                supply_total_sats,
                |(height, profit_sats, volatility, total_sats, ..)| {
                    let total = total_sats.as_u128() as f64;
                    if total == 0.0 {
                        (height, Ratio::new(0.0))
                    } else {
                        let pct_in_profit = profit_sats.as_u128() as f64 / total;
                        (
                            height,
                            Ratio::new((pct_in_profit * f64::from(volatility) / 100.0) as f32),
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

        Ok(())
    }
}

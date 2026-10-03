use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use rayon::join;
use vecdb::{AnyVec, Database};

use super::{Vecs, activity, adjusted, age_range, aggregate, cap, prices, reserve_risk, value};
use crate::Dependencies;

impl Vecs {
    /// Compute the scalar inputs for this plugin's URPD metrics.
    fn compute_primary(
        &mut self,
        dependencies: Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            indexer,
            urpd: _,
            price: prices,
            blocks,
            inflation_rate,
            velocity_native,
            velocity_fiat,
            distribution_age,
            distribution_aggregated,
        } = dependencies;
        let inflation_rate = &inflation_rate.ppm.height;
        let velocity_native = &velocity_native.height;
        let velocity_fiat = &velocity_fiat.height;
        let exit = context.exit();

        // Activity computes first (liveliness, vaultedness, etc.)
        activity::compute(
            &mut self.activity,
            indexer,
            distribution_age,
            distribution_aggregated,
            exit,
        )?;
        age_range::compute(&mut self.age_range, indexer, distribution_age, exit)?;

        // Age-range supply is lazy over the same cached inputs as aggregates.
        // Adjusted and value compute independently.
        let (r1, r2) = join(
            || {
                aggregate::compute(
                    &mut self.aggregate,
                    indexer,
                    distribution_age,
                    &mut self.age_range,
                    &mut self.supply.active_supply_in_loss_share.bounded,
                    exit,
                )
            },
            || {
                join(
                    || {
                        adjusted::compute(
                            &mut self.adjusted,
                            indexer,
                            inflation_rate,
                            velocity_native,
                            velocity_fiat,
                            &self.activity,
                            exit,
                        )
                    },
                    || {
                        value::compute(
                            &mut self.value,
                            indexer,
                            prices,
                            distribution_age,
                            distribution_aggregated,
                            &self.activity,
                            exit,
                        )
                    },
                )
            },
        );
        r1?;
        r2.0?;
        r2.1?;

        // Cap depends on activity + value
        cap::compute(
            &mut self.cap,
            indexer,
            distribution_aggregated,
            &self.activity,
            &self.value,
            exit,
        )?;

        // Phase 4: pricing and reserve_risk are independent
        let (r3, r4) = join(
            || {
                prices::compute(
                    &mut self.prices,
                    indexer,
                    distribution_aggregated,
                    &self.activity,
                    &self.supply,
                    &self.cap,
                    exit,
                )
            },
            || {
                reserve_risk::compute(
                    &mut self.reserve_risk,
                    indexer,
                    blocks,
                    prices,
                    &self.value,
                    exit,
                )
            },
        );
        r3?;
        r4?;

        Ok(())
    }
}

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
        self.compute_primary(dependencies, context)?;
        let supplies = dependencies
            .distribution_age
            .cohorts
            .supply
            .total
            .age_supplies();
        let weights = self.age_range.urpd_weight_sources();
        self.urpd.compute(
            dependencies.distribution_age.cohorts.all_supply().version()
                + dependencies.urpd.timestamps.version(),
            usize::from(dependencies.indexer.safe_lengths().height),
            dependencies.urpd,
            &weights,
            &supplies,
            context.exit(),
        )?;
        Ok(())
    }
}

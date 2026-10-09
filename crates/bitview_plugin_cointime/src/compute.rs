use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use rayon::join;
use vecdb::{AnyVec, Database};

use super::Vecs;
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
            age,
            holders,
        } = dependencies;
        let inflation_rate = &inflation_rate.fixed.height;
        let velocity_native = &velocity_native.height;
        let velocity_fiat = &velocity_fiat.height;
        let exit = context.exit();

        // Activity computes first (liveliness, vaultedness, etc.)
        self.activity.compute(indexer, age, holders, exit)?;
        self.age_range.compute(indexer, age, exit)?;

        // Age-range supply is lazy over the same cached inputs as aggregates.
        // Adjusted and value compute independently.
        let (r1, r2) = join(
            || {
                self.aggregate.compute(
                    indexer,
                    age,
                    &mut self.age_range,
                    &mut self.supply.active_supply_in_loss_share.fixed,
                    exit,
                )
            },
            || {
                join(
                    || {
                        self.adjusted.compute(
                            indexer,
                            inflation_rate,
                            velocity_native,
                            velocity_fiat,
                            &self.activity,
                            exit,
                        )
                    },
                    || {
                        self.value
                            .compute(indexer, prices, age, holders, &self.activity, exit)
                    },
                )
            },
        );
        r1?;
        r2.0?;
        r2.1?;

        // Cap depends on activity + value
        self.cap
            .compute(indexer, holders, &self.activity, &self.value, exit)?;

        // Phase 4: pricing and reserve_risk are independent
        let (r3, r4) = join(
            || {
                self.prices.compute(
                    indexer,
                    holders,
                    &self.activity,
                    &self.supply,
                    &self.cap,
                    exit,
                )
            },
            || {
                self.reserve_risk
                    .compute(indexer, blocks, prices, &self.value, exit)
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
        let supplies = dependencies.age.cohorts.supply.total.age_supplies();
        let weights = self.age_range.urpd_weight_sources();
        self.urpd.compute(
            dependencies.age.cohorts.all_supply().version()
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

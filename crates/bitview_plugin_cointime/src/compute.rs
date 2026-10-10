use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_urpd::compute_cost_basis;
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
            mappings,
            price: prices,
            blocks,
            inflation_rate,
            velocity_btc,
            velocity_usd,
            age,
            holders,
        } = dependencies;
        let inflation_rate = &inflation_rate.fixed.height;
        let velocity_btc = &velocity_btc.height;
        let velocity_usd = &velocity_usd.height;
        let exit = context.exit();

        // Activity computes first (liveliness, vaultedness, etc.)
        self.activity.compute(indexer, age, holders, exit)?;
        self.age_ranges.compute(indexer, age, exit)?;

        // Age-range supply is lazy over the same cached inputs as aggregates.
        // Adjusted and value compute independently.
        let (r1, r2) = join(
            || self.aggregate.compute(indexer, age, &self.age_ranges, exit),
            || {
                join(
                    || {
                        self.adjusted.compute(
                            indexer,
                            inflation_rate,
                            velocity_btc,
                            velocity_usd,
                            &self.activity,
                            exit,
                        )
                    },
                    || {
                        self.value
                            .compute(indexer, prices, age, &self.activity, exit)
                    },
                )
            },
        );
        r1?;
        r2.0?;
        r2.1?;

        // Cap depends on activity + value
        self.caps
            .compute(indexer, holders, &self.activity, &self.value, exit)?;

        // Phase 4: pricing and reserve_risk are independent
        let (r3, r4) = join(
            || {
                let from = indexer.safe_lengths().height;
                self.prices.compute(
                    from,
                    holders,
                    &self.activity,
                    &self.supply,
                    &self.caps,
                    exit,
                )?;
                self.prices
                    .compute_ratios(from, &prices.spot.cents.height, exit)
            },
            || {
                self.reserve_risk
                    .compute(indexer, blocks, mappings, prices, holders, exit)
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
        let supplies = dependencies.age.age_supplies();
        let weights = self.age_ranges.urpd_weight_sources();
        compute_cost_basis(
            &mut self.urpd_replay,
            self.aggregate
                .cohorts
                .as_array_mut()
                .map(|cohort| &mut cohort.awake.cost_basis),
            dependencies.age.all_supply().version() + dependencies.urpd.timestamps.version(),
            usize::from(dependencies.indexer.safe_lengths().height),
            dependencies.urpd,
            &weights,
            &supplies,
            context.exit(),
        )?;
        let from = dependencies.indexer.safe_lengths().height;
        let spot = &dependencies.price.spot.cents.height;
        for cohort in self.aggregate.cohorts.iter_mut() {
            cohort
                .awake
                .cost_basis
                .compute_ratios(from, spot, context.exit())?;
        }
        Ok(())
    }
}

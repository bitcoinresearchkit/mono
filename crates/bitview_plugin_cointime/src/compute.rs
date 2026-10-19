use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_urpd::compute_cost_basis;
use brk_error::Result;
use rayon::join;
use vecdb::AnyVec;

use super::Vecs;
use crate::Dependencies;

impl Vecs {
    /// Everything but the awake cost basis, which the age ranges computed here weight.
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

        // Reserve risk reads only dependencies: it runs beside the whole chain below.
        let (primary, reserve_risk) = join(
            || -> Result<()> {
                // Activity (liveliness, vaultedness, etc.) and the age ranges are independent.
                let (activity, age_ranges) = join(
                    || self.activity.compute(indexer, age, holders, exit),
                    || self.age_ranges.compute(indexer, age, exit),
                );
                activity?;
                age_ranges?;

                // Age-range supply is lazy over the same cached inputs as aggregates.
                // Adjusted and value compute independently.
                let (aggregate, (adjusted, value)) = join(
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
                aggregate?;
                adjusted?;
                value?;

                // Cap depends on activity + value, pricing on cap.
                self.caps
                    .compute(indexer, holders, &self.activity, &self.value, exit)?;
                let from = indexer.compute_start().height;
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
                    .compute(indexer, mappings, prices, holders, exit)
            },
        );
        primary?;
        reserve_risk?;

        Ok(())
    }
}

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

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
            usize::from(dependencies.indexer.compute_start().height),
            dependencies.urpd,
            &weights,
            &supplies,
            &dependencies.price.spot.cents.height,
            context.exit(),
        )
    }
}

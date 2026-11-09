use bitview_distribution::urpd_metrics::compute_cost_basis;
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_primitives::Lengths;
use brk_error::Result;
use brk_exit::Exit;
use rayon::join;
use vecdb::AnyVec;

use super::Vecs;
use crate::Dependencies;

impl Vecs {
    /// Everything but the awake cost basis, which the age ranges computed here weight.
    fn compute_primary(
        &mut self,
        dependencies: Dependencies<'_>,
        starting_lengths: Lengths,
        exit: &Exit,
    ) -> Result<()> {
        let Dependencies {
            urpd: _,
            mappings,
            price: prices,
            inflation_rate,
            volume_1y,
            age,
            holders,
        } = dependencies;
        let inflation_rate = &inflation_rate.fixed.height;
        let all_chain = holders.all_chain_sources();

        // Reserve risk reads only dependencies: it runs beside the whole chain below.
        let (primary, reserve_risk) = join(
            || -> Result<()> {
                // Activity (liveliness, vaultedness, etc.) and the age ranges are independent.
                let (activity, age_ranges) = join(
                    || {
                        self.activity
                            .compute(age, holders, starting_lengths.height, exit)
                    },
                    || self.age_ranges.compute(age, starting_lengths.height, exit),
                );
                activity?;
                age_ranges?;

                // Age-range supply is lazy over the same cached inputs as aggregates.
                // Adjusted and value compute independently.
                let (aggregate, (adjusted, value)) = join(
                    || {
                        self.aggregate
                            .compute(age, &self.age_ranges, starting_lengths.height, exit)
                    },
                    || {
                        join(
                            || {
                                self.adjusted.compute(
                                    inflation_rate,
                                    volume_1y,
                                    &all_chain,
                                    &self.activity,
                                    starting_lengths.height,
                                    exit,
                                )
                            },
                            || {
                                self.value.compute(
                                    prices,
                                    age,
                                    &self.activity,
                                    starting_lengths.height,
                                    exit,
                                )
                            },
                        )
                    },
                );
                aggregate?;
                adjusted?;
                value?;

                // Cap depends on activity + value, pricing on cap.
                self.caps.compute(
                    holders,
                    &self.activity,
                    &self.value,
                    starting_lengths,
                    exit,
                )?;
                let from = starting_lengths.height;
                self.prices
                    .compute(from, holders, &self.supply, &self.caps, exit)?;
                self.prices
                    .compute_ratios(from, &prices.spot.cents.height, exit)
            },
            || {
                self.reserve_risk
                    .compute(mappings, prices, holders, starting_lengths.height, exit)
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
        let starting_lengths = context.start();
        self.compute_primary(dependencies, starting_lengths, context.exit())?;
        let supplies = dependencies.age.age_supplies();
        let weights = self.age_ranges.urpd_weight_sources();
        compute_cost_basis(
            &mut self.urpd_replay,
            self.aggregate
                .cohorts
                .as_array_mut()
                .map(|cohort| &mut cohort.awake.cost_basis),
            dependencies.age.all_supply().version() + dependencies.urpd.timestamps.version(),
            usize::from(starting_lengths.height),
            dependencies.urpd,
            &weights,
            &supplies,
            &dependencies.price.spot.cents.height,
            context.exit(),
        )
    }
}

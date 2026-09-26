use bitview_cohort::AgeRange;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_urpd::DailyUrpds;
use brk_error::Result;
use rayon::join;
use vecdb::AnyVec;

use super::{Vecs, activity, adjusted, age_range, aggregate, cap, prices, reserve_risk, value};
use crate::Dependencies;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        let Dependencies {
            age_urpds,
            mappings,
            indexer,
            price: prices,
            blocks,
            inflation_rate,
            velocity_native,
            velocity_fiat,
            distribution,
        } = dependencies;
        let inflation_rate = &inflation_rate.ppm.height;
        let velocity_native = &velocity_native.height;
        let velocity_fiat = &velocity_fiat.height;
        let exit = context.exit();

        self.db.sync_bg_tasks()?;

        // Activity computes first (liveliness, vaultedness, etc.)
        activity::compute(&mut self.activity, indexer, distribution, exit)?;
        age_range::compute(&mut self.age_range, indexer, distribution, exit)?;

        // Age-range supply is lazy over the same cached inputs as aggregates.
        // Adjusted and value compute independently.
        let (r1, r2) = join(
            || {
                aggregate::compute(
                    &mut self.aggregate,
                    indexer,
                    distribution,
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
                            distribution,
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
            distribution,
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
                    distribution,
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

        let weights =
            AgeRange::from_fn(|age| &age.select(&self.age_range.activity.wakefulness).day1);
        let supplies = AgeRange::from_fn(|age| {
            &age.select(&distribution.cohorts.supply.total.cohorts.utxo.age)
                .sats
                .day1
        });
        let from = mappings
            .height
            .recompute_day(indexer.safe_lengths().height)
            .map(usize::from)
            .unwrap_or_default();
        self.urpd.compute(
            distribution.supply_state.version(),
            from,
            &mappings.day1.date,
            &prices.split.close.cents.day1,
            &weights,
            &supplies,
            |day, date, weights| {
                age_urpds.with_entries(
                    &distribution.states_path,
                    date,
                    usize::from(day) + 1 == mappings.day1.date.len(),
                    |entries| DailyUrpds::from_age_entries(entries, weights),
                )
            },
            exit,
        )?;

        context.compact_database(&self.db);

        Ok(())
    }
}

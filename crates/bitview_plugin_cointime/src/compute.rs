use brk_error::Result;

use bitview_plugin::ComputePlugin;
use brk_indexer::Indexer;
use brk_types::{Height, PartsPerMillionSigned64, StoredF64};
use vecdb::{Exit, ReadableVec};

use super::Vecs;

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        prices: &bitview_plugin_price::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        inflation_rate: &impl ReadableVec<Height, PartsPerMillionSigned64>,
        velocity_native: &impl ReadableVec<Height, StoredF64>,
        velocity_fiat: &impl ReadableVec<Height, StoredF64>,
        distribution: &bitview_plugin_distribution::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        // Activity computes first (liveliness, vaultedness, etc.)
        super::activity::compute(&mut self.activity, indexer, distribution, exit)?;
        super::age_range::compute(&mut self.age_range, indexer, distribution, exit)?;

        // Phase 2: age-weighted aggregates, adjusted, and value are independent.
        let (r1, r2) = rayon::join(
            || {
                super::aggregate::compute(
                    &mut self.aggregate,
                    indexer,
                    distribution,
                    &self.age_range,
                    &mut self.supply.active_supply_in_loss_share,
                    exit,
                )
            },
            || {
                rayon::join(
                    || {
                        super::adjusted::compute(
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
                        super::value::compute(
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
        super::cap::compute(
            &mut self.cap,
            indexer,
            distribution,
            &self.activity,
            &self.value,
            exit,
        )?;

        // Phase 4: pricing and reserve_risk are independent
        let (r3, r4) = rayon::join(
            || {
                super::prices::compute(
                    &mut self.prices,
                    indexer,
                    prices,
                    distribution,
                    &self.activity,
                    &self.supply,
                    &self.cap,
                    exit,
                )
            },
            || {
                super::reserve_risk::compute(
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

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });

        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = crate::Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        exit: &Exit,
    ) -> Result<Self::Output> {
        self.compute_inner(
            dependencies.indexer,
            dependencies.price,
            dependencies.blocks,
            &dependencies.inflation_rate.ppm.height,
            &dependencies.velocity_native.height,
            &dependencies.velocity_fiat.height,
            dependencies.distribution,
            exit,
        )
    }
}

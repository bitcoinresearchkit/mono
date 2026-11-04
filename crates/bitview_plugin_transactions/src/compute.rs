use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_primitives::Count;
use brk_error::Result;
use rayon::join;

use super::Vecs;
use crate::{Dependencies, fees::Valued};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            indexer,
            mappings,
            price,
        } = dependencies;
        let Vecs {
            count,
            features,
            vsize,
            fees,
            inscription,
            patterns,
            nonstandard,
            versions,
            transfer_volume,
            ..
        } = self;
        let exit = context.exit();
        let start = indexer.compute_start().height;
        let price = &price.spot.cents.height;

        // The counts, from the indexer's, apart from the values the fee pass gives the same kinds.
        let (feature_counts, feature_values) = features.split_mut();
        let (version_counts, version_values) = versions.split_mut();

        let (independent, fee_based) = join(
            // Stages reading only indexer and mappings columns.
            || {
                let ((count, patterns), (features, (versions, inscription))) = join(
                    || {
                        join(
                            || count.compute(start, &mappings.lookback.window_starts(), exit),
                            || patterns.compute(indexer, mappings, exit),
                        )
                    },
                    || {
                        join(
                            || feature_counts.compute(indexer, exit),
                            || {
                                join(
                                    || version_counts.compute(indexer, exit),
                                    || {
                                        inscription.count.count.compute_cumulative_transformed(
                                            start,
                                            &*indexer
                                                .vecs()
                                                .transactions
                                                .features
                                                .inscription
                                                .block,
                                            Count::from,
                                            exit,
                                        )
                                    },
                                )
                            },
                        )
                    },
                );
                count
                    .and(patterns)
                    .and(features)
                    .and(versions)
                    .and(inscription)
            },
            // The fee pass (with the vsize distribution and every kind's value), then dollars.
            || {
                let valued = Valued {
                    features: feature_values,
                    inscription: &mut inscription.count.value.value,
                    versions: version_values,
                    nonstandard: &mut nonstandard.count.value.value,
                    nonstandard_flag: &mut nonstandard.flag,
                    nonstandard_count: &mut nonstandard.count.count,
                    inscription_fees: &mut inscription.fees.cumulative.sats.height,
                };
                fees.compute(
                    indexer,
                    mappings,
                    &mut vsize.distribution,
                    &mut transfer_volume.cumulative.sats.height,
                    valued,
                    exit,
                )?;
                let (inscription, volume) = join(
                    || inscription.fees.compute_cents(start, price, exit),
                    || transfer_volume.compute_cents(start, price, exit),
                );
                inscription.and(volume)
            },
        );
        independent.and(fee_based)
    }
}

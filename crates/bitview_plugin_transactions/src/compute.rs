use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_primitives::Count;
use brk_error::Result;
use rayon::{join, prelude::*};

use super::Vecs;
use crate::{
    Dependencies,
    fees::{Tallied, Valued},
    flagged::TxValue,
};

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
            payments,
            timelocks,
            spending,
            ephemeral_dust,
            same_block_spending,
            ..
        } = self;
        let exit = context.exit();
        let start = indexer.compute_start().height;
        let price = &price.spot.cents.height;

        // The counts, from the indexer's, apart from the values the fee pass gives the same kinds.
        let (feature_counts, feature_values, shapes, bip69_expected) = features.split_mut();
        let (version_counts, version_values) = versions.split_mut();

        let (independent, fee_based) = join(
            // Stages reading only indexer and mappings columns.
            || {
                let ((count, patterns), (features, (versions, inscription))) = join(
                    || {
                        join(
                            || count.compute(start, &mappings.lookback.window_starts(), exit),
                            || {
                                patterns.compute(indexer, mappings, exit)?;
                                patterns.kinds_mut().into_iter().try_for_each(|kind| {
                                    kind.value.compute_cents(start, price, exit)
                                })
                            },
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
                let mut valued = Valued {
                    features: feature_values,
                    inscription: &mut inscription.count.value,
                    versions: version_values,
                    nonstandard: &mut nonstandard.count.value,
                    nonstandard_flag: &mut nonstandard.flag,
                    nonstandard_count: &mut nonstandard.count.count,
                    inscription_fees: &mut inscription.fees.cumulative.sats.height,
                    vsize_total: &mut vsize.total,
                    bip69_expected,
                    tallied: Tallied {
                        one_input_one_output: &mut shapes.one_input_one_output,
                        one_input_two_output: &mut shapes.one_input_two_output,
                        payment: &mut payments.count,
                        segwit_payment: &mut payments.segwit_spending,
                        rbf_payment: &mut payments.rbf_signaling,
                        ephemeral_dust,
                        same_block_spending,
                    },
                };
                fees.compute(
                    indexer,
                    mappings,
                    &mut vsize.distribution,
                    &mut transfer_volume.cumulative.sats.height,
                    &mut valued,
                    exit,
                )?;
                // Each block's dollars, its sats at its price, every kind's at once.
                let mut values: Vec<&mut TxValue> =
                    valued.values_mut().map(|(_, value)| value).collect();
                values.extend([
                    &mut fees.cpfp_parent.count.value,
                    &mut fees.cpfp_child.count.value,
                ]);
                let (kinds, volume) = join(
                    || {
                        values
                            .par_iter_mut()
                            .try_for_each(|value| value.compute_cents(start, price, exit))
                    },
                    || transfer_volume.compute_cents(start, price, exit),
                );
                kinds.and(volume)?;
                inscription.fees.compute_cents(start, price, exit)
            },
        );
        independent.and(fee_based)?;

        let counters = &indexer.vecs().transactions.counters;
        shapes
            .bip69
            .compute(start, &counters.bip69_tx_count, exit)?;
        timelocks.compute(start, indexer, exit)?;
        spending.compute(start, counters, exit)
    }
}

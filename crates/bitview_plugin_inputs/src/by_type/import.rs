use bitview_cohort::{SpendableType, type_key};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_transforms::Quotient;
use bitview_vecs::{
    LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling, LazyWindowStartVec, import_cached,
};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, LazyVec, ReadableCloneableVec};

use super::{InputTypeVecs, Vecs};
use crate::count::without_coinbase;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        inputs: &impl ReadableCloneableVec<Height, Count>,
    ) -> Result<Self> {
        let version = version + Version::TWO;
        let count_stored = SpendableType::try_new(|id| {
            import_cached(
                db,
                &format!("{}_prevout_count_cumulative", id.name()),
                version,
            )
        })?;
        let tx_count_stored = SpendableType::try_new(|id| {
            import_cached(
                db,
                &format!("tx_count_with_{}_prevout_cumulative", id.name()),
                version,
            )
        })?;
        // Coinbase transactions spend no outputs.
        let txs = LazyVec::init(
            "non_coinbase_tx_count_cumulative_source",
            version,
            mappings.transaction_count_source().read_only_boxed_clone(),
            without_coinbase,
        );
        let version = version + Version::ONE;
        let types = SpendableType::from_fn(|kind| {
            let output_type = kind.output_type();
            let key = type_key(output_type);
            let count = LazyPerBlockCumulativeRolling::from_cumulative_source(
                &format!("{key}_input_count"),
                version,
                count_stored.get(output_type),
                window_starts,
                mappings,
            );
            let share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
                Count,
                Count,
                Quotient<PartsPerMillion32>,
            >(
                &format!("{key}_input_share"),
                version,
                &count.cumulative.height,
                inputs,
                window_starts,
                mappings,
            );
            let tx_count = LazyPerBlockCumulativeRolling::from_cumulative_source(
                &format!("{key}_input_tx_count"),
                version,
                tx_count_stored.get(output_type),
                window_starts,
                mappings,
            );
            let tx_share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
                Count,
                Count,
                Quotient<PartsPerMillion32>,
            >(
                &format!("{key}_input_tx_share"),
                version,
                &tx_count.cumulative.height,
                &txs,
                window_starts,
                mappings,
            );
            InputTypeVecs {
                count,
                share,
                tx_count,
                tx_share,
            }
        });

        Ok(Self {
            types,
            count_stored,
            tx_count_stored,
        })
    }
}

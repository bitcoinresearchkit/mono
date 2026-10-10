use bitview_cohort::SpendableType;
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
        let count_stored = SpendableType::try_from_fn(|kind| {
            import_cached(
                db,
                &format!("{}_prevout_count_cumulative", kind.key()),
                version,
            )
        })?;
        let tx_count_stored = SpendableType::try_from_fn(|kind| {
            import_cached(
                db,
                &format!("tx_count_with_{}_prevout_cumulative", kind.key()),
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
            let key = kind.key();
            let count = LazyPerBlockCumulativeRolling::from_cumulative_source(
                &format!("{key}_input_count"),
                version,
                kind.select(&count_stored),
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
                kind.select(&tx_count_stored),
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

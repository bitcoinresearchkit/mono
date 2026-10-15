use bitview_cohort::SpendableType;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{CumulativeSource, LazyWindowStartVec, count_and_share, shapes::Windows};
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
            CumulativeSource::import(
                db,
                &format!("{}_prevout_count_cumulative", kind.key()),
                version,
            )
        })?;
        let tx_count_stored = SpendableType::try_from_fn(|kind| {
            CumulativeSource::import(
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
            let (count, share) = count_and_share(
                &format!("{key}_input"),
                version,
                kind.select(&count_stored).cumulative_source(),
                inputs,
                window_starts,
                mappings,
            );
            let (tx_count, tx_share) = count_and_share(
                &format!("{key}_input_tx"),
                version,
                kind.select(&tx_count_stored).cumulative_source(),
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

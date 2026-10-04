use bitview_cohort::SpendableType;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{CountTotal, LazyWindowStartVec, import_cached};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::Database;

use super::{Vecs, WithInputTypes};

fn without_coinbase(height: Height, total: Count) -> Count {
    total - Count::from(height.incremented())
}

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::TWO;
        let input_count_stored = SpendableType::try_new(|id| {
            import_cached(
                db,
                &format!("{}_prevout_count_cumulative", id.name()),
                version,
            )
        })?;
        let input_count = WithInputTypes::from_cumulative_sources(
            CountTotal::from_source(
                "input_count_bis",
                version,
                &mappings.input_count_source(),
                mappings,
                window_starts,
            ),
            |name| format!("{name}_prevout_count"),
            version,
            &input_count_stored,
            mappings,
            window_starts,
        );
        let input_share = input_count.lazy_shares(
            version,
            |name| format!("{name}_prevout_share"),
            window_starts,
            mappings,
        );
        let tx_count_stored = SpendableType::try_new(|id| {
            import_cached(
                db,
                &format!("tx_count_with_{}_prevout_cumulative", id.name()),
                version,
            )
        })?;
        let tx_count = WithInputTypes::from_cumulative_sources(
            CountTotal::from_transformed_source(
                "non_coinbase_tx_count",
                version,
                &mappings.transaction_count_source(),
                without_coinbase,
                mappings,
                window_starts,
            ),
            |name| format!("tx_count_with_{name}_prevout"),
            version,
            &tx_count_stored,
            mappings,
            window_starts,
        );
        let tx_share = tx_count.lazy_shares(
            version,
            |name| format!("tx_share_with_{name}_prevout"),
            window_starts,
            mappings,
        );

        Ok(Self {
            input_count,
            input_share,
            tx_count,
            tx_share,
            input_count_stored,
            tx_count_stored,
        })
    }
}

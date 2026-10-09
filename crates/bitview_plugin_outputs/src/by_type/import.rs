use bitview_cohort::{ByType, type_key};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_transforms::Quotient;
use bitview_vecs::{
    LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling, LazyWindowStartVec, import_cached,
};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{OutputTypeVecs, Vecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::TWO;
        let count_stored = ByType::try_new(|id| {
            import_cached(
                db,
                &format!("{}_output_count_cumulative", id.name()),
                version,
            )
        })?;
        let tx_count_stored = ByType::try_new(|id| {
            import_cached(
                db,
                &format!("tx_count_with_{}_output_cumulative", id.name()),
                version,
            )
        })?;
        let outputs = mappings.output_count_source();
        let txs = mappings.transaction_count_source();
        let types = ByType::from_type(|output_type| {
            let key = type_key(output_type);
            let count = LazyPerBlockCumulativeRolling::from_cumulative_source(
                &format!("{key}_output_count"),
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
                &format!("{key}_output_share"),
                version,
                &count.cumulative.height,
                &outputs,
                window_starts,
                mappings,
            );
            let tx_count = LazyPerBlockCumulativeRolling::from_cumulative_source(
                &format!("{key}_output_tx_count"),
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
                &format!("{key}_output_tx_share"),
                version,
                &tx_count.cumulative.height,
                &txs,
                window_starts,
                mappings,
            );
            OutputTypeVecs {
                count,
                share,
                tx_count,
                tx_share,
            }
        });

        Ok(Vecs {
            types,
            count_stored,
            tx_count_stored,
        })
    }
}

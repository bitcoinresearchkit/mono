use bitview_cohort::ByType;
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
        let count_stored = ByType::try_from_fn(|kind| {
            import_cached(
                db,
                &format!("{}_output_count_cumulative", kind.key()),
                version,
            )
        })?;
        let tx_count_stored = ByType::try_from_fn(|kind| {
            import_cached(
                db,
                &format!("tx_count_with_{}_output_cumulative", kind.key()),
                version,
            )
        })?;
        let outputs = mappings.output_count_source();
        let txs = mappings.transaction_count_source();
        let types = ByType::from_fn(|kind| {
            let key = kind.key();
            let count = LazyPerBlockCumulativeRolling::from_cumulative_source(
                &format!("{key}_output_count"),
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
                kind.select(&tx_count_stored),
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

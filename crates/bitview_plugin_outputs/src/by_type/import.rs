use bitview_cohort::ByType;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{CountTotal, LazyWindowStartVec, import_cached};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{SpendableOutputCount, Vecs, WithOutputTypes};

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::TWO;
        let output_count_stored = ByType::try_new(|id| {
            import_cached(
                db,
                &format!("{}_output_count_cumulative", id.name()),
                version,
            )
        })?;
        let output_count = WithOutputTypes::from_cumulative_sources(
            CountTotal::from_source(
                "output_count_bis",
                version,
                &mappings.output_count_source(),
                mappings,
                window_starts,
            ),
            |name| format!("{name}_output_count"),
            version,
            &output_count_stored,
            mappings,
            window_starts,
        );
        let output_share = output_count.lazy_shares(
            version,
            |name| format!("{name}_output_share"),
            window_starts,
            mappings,
        );
        let tx_count_stored = ByType::try_new(|id| {
            import_cached(
                db,
                &format!("tx_count_with_{}_output_cumulative", id.name()),
                version,
            )
        })?;
        let tx_count = WithOutputTypes::from_cumulative_sources(
            CountTotal::from_source(
                "tx_count_bis",
                version,
                &mappings.transaction_count_source(),
                mappings,
                window_starts,
            ),
            |name| format!("tx_count_with_{name}_output"),
            version,
            &tx_count_stored,
            mappings,
            window_starts,
        );
        let tx_share = tx_count.lazy_shares(
            version,
            |name| format!("tx_share_with_{name}_output"),
            window_starts,
            mappings,
        );

        let op_return_count = &output_count.by_type.unspendable.op_return.cumulative.height;
        let spendable_output_count =
            SpendableOutputCount::new(version, op_return_count, mappings, window_starts);

        Ok(Vecs {
            output_count,
            output_share,
            tx_count,
            tx_share,
            output_count_stored,
            tx_count_stored,
            spendable_output_count,
        })
    }
}

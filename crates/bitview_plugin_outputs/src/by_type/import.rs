use brk_error::Result;

use bitview_cohort::OutputTypeId;
use brk_types::{Height, StoredU16, StoredU64, Version};
use vecdb::Database;

use super::{CachedSpendableOutputCount, Vecs, WithOutputTypes};
use bitview_compute::{
    CachedWindowStartVec, ColumnarPerBlock, ColumnarPerBlockCumulativeRolling, Windows,
};

fn identity(_: Height, value: StoredU64) -> StoredU64 {
    value
}

pub fn forced_import(
    db: &Database,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
    cached_starts: &Windows<&CachedWindowStartVec>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, mappings, cached_starts)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let columnar_version = version + Version::ONE;
        let all_output_count = mappings.output_count_source();
        let output_count = ColumnarPerBlock::<StoredU16, OutputTypeId, _>::forced_import(
            db,
            "output_count_by_type",
            columnar_version,
            |source| {
                WithOutputTypes::from_columnar_count_source(
                    "output_count_bis",
                    |t| format!("{t}_output_count"),
                    columnar_version,
                    (all_output_count, identity),
                    source,
                    mappings,
                    cached_starts,
                )
            },
        )?;
        let output_share = output_count.lazy_shares(
            columnar_version,
            |name| format!("{name}_output_share"),
            cached_starts,
            mappings,
        );
        let all_tx_count = mappings.transaction_count_source();
        let tx_count =
            ColumnarPerBlockCumulativeRolling::<StoredU64, OutputTypeId, _>::forced_import(
                db,
                "tx_count_with_output_by_type_cumulative",
                columnar_version,
                |source| {
                    WithOutputTypes::from_columnar_source(
                        "tx_count_bis",
                        |t| format!("tx_count_with_{t}_output"),
                        columnar_version,
                        (all_tx_count, identity),
                        source,
                        mappings,
                        cached_starts,
                    )
                },
            )?;
        let tx_share = tx_count.lazy_shares(
            columnar_version,
            |name| format!("tx_share_with_{name}_output"),
            cached_starts,
            mappings,
        );

        let op_return_count = output_count
            .by_type
            .unspendable
            .op_return
            .cached_cumulative();
        let spendable_output_count =
            CachedSpendableOutputCount::new(version, &op_return_count, mappings, cached_starts);

        Ok(Self {
            output_count,
            spendable_output_count,
            output_share,
            tx_count,
            tx_share,
        })
    }
}

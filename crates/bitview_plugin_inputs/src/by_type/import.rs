use brk_error::Result;

use bitview_compute::{
    CachedWindowStartVec, ColumnarPerBlock, ColumnarPerBlockCumulativeRolling, Windows,
};
use brk_cohort::SpendableTypeId;
use brk_types::{Height, StoredU16, StoredU64, Version};
use vecdb::Database;

use super::{Vecs, WithInputTypes};

fn identity(_: Height, value: StoredU64) -> StoredU64 {
    value
}

fn without_coinbase(height: Height, total: StoredU64) -> StoredU64 {
    total - StoredU64::from(height.incremented())
}

impl Vecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let columnar_version = version + Version::ONE;
        let all_input_count = indexes.input_count_source();
        let input_count = ColumnarPerBlock::<StoredU16, SpendableTypeId, _>::forced_import(
            db,
            "prevout_count_by_type",
            columnar_version,
            |source| {
                WithInputTypes::from_columnar_count_source(
                    "input_count_bis",
                    |t| format!("{t}_prevout_count"),
                    columnar_version,
                    (all_input_count, identity),
                    source,
                    indexes,
                    cached_starts,
                )
            },
        )?;
        let input_share = input_count.lazy_shares(
            columnar_version,
            |name| format!("{name}_prevout_share"),
            cached_starts,
            indexes,
        );
        let transaction_count_source = indexes.transaction_count_source();
        let tx_count =
            ColumnarPerBlockCumulativeRolling::<StoredU64, SpendableTypeId, _>::forced_import(
                db,
                "tx_count_with_prevout_by_type_cumulative",
                columnar_version,
                |source| {
                    WithInputTypes::from_columnar_source(
                        "non_coinbase_tx_count",
                        |t| format!("tx_count_with_{t}_prevout"),
                        columnar_version,
                        (transaction_count_source, without_coinbase),
                        source,
                        indexes,
                        cached_starts,
                    )
                },
            )?;
        let tx_share = tx_count.lazy_shares(
            columnar_version,
            |name| format!("tx_share_with_{name}_prevout"),
            cached_starts,
            indexes,
        );
        Ok(Self {
            input_count,
            input_share,
            tx_count,
            tx_share,
        })
    }
}

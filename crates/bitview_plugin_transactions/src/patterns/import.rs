use brk_error::Result;

use bitview_compute::{
    CachedWindowStartVec, ColumnarPerBlockCumulativeRolling, LazyColumnPerBlockCumulativeRolling,
    Windows,
};
use brk_types::{StoredBool, TxIndex, Version};
use vecdb::{
    ColumnarVec, Database, EagerVec, ImportableVec, PcoVec, ReadOnlyClone, ReadableColumnarVec,
};

use super::{CountVecs, Flags, PatternId, Vecs};

pub fn forced_import(
    db: &Database,
    version: Version,
    indexes: &bitview_plugin_indexes::Vecs,
    cached_starts: &Windows<&CachedWindowStartVec>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, indexes, cached_starts)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let count_source = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            "transaction_pattern_count_cumulative",
            version,
            |_| (),
        )?;
        let counts = count_source.cumulative.read_only_clone();
        let count = CountVecs {
            coinjoin: LazyColumnPerBlockCumulativeRolling::new(
                "coinjoin_count",
                version,
                &counts,
                PatternId::Coinjoin,
                indexes,
                cached_starts,
            ),
            consolidation: LazyColumnPerBlockCumulativeRolling::new(
                "consolidation_count",
                version,
                &counts,
                PatternId::Consolidation,
                indexes,
                cached_starts,
            ),
            batch_payout: LazyColumnPerBlockCumulativeRolling::new(
                "batch_payout_count",
                version,
                &counts,
                PatternId::BatchPayout,
                indexes,
                cached_starts,
            ),
            source: count_source,
        };

        let flags_source =
            EagerVec::<ColumnarVec<PcoVec<TxIndex, StoredBool>, PatternId>>::forced_import(
                db,
                "transaction_pattern_flags",
                version,
            )?;
        let flags = flags_source.read_only_clone();

        Ok(Self {
            count,
            flags: Flags {
                is_coinjoin: flags.column("is_coinjoin", version, PatternId::Coinjoin),
                is_consolidation: flags.column(
                    "is_consolidation",
                    version,
                    PatternId::Consolidation,
                ),
                is_batch_payout: flags.column("is_batch_payout", version, PatternId::BatchPayout),
            },
            flags_source,
        })
    }
}

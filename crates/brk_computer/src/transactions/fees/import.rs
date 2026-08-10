use brk_error::Result;
use brk_types::{StoredBool, TxIndex, Version};
use vecdb::{
    ColumnarVec, Database, EagerVec, ImportableVec, PcoVec, ReadOnlyClone, ReadableColumnarVec,
};

use super::{CountVecs, CpfpFlags, CpfpRoleId, Vecs};
use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarPerBlockCumulativeRolling,
        LazyColumnPerBlockCumulativeRolling, PerTxDistribution, Windows,
    },
};

/// Bump this when fee/feerate aggregation logic changes (e.g., skip coinbase, skip zero-fee).
const VERSION: Version = Version::new(3);

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let v = version + VERSION;
        let count_source = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            "cpfp_count_cumulative",
            version,
            |_| (),
        )?;
        let counts = count_source.cumulative.read_only_clone();
        let count = CountVecs {
            cpfp_parent: LazyColumnPerBlockCumulativeRolling::new(
                "cpfp_parent_count",
                version,
                &counts,
                CpfpRoleId::Parent,
                indexes,
                cached_starts,
            ),
            cpfp_child: LazyColumnPerBlockCumulativeRolling::new(
                "cpfp_child_count",
                version,
                &counts,
                CpfpRoleId::Child,
                indexes,
                cached_starts,
            ),
            source: count_source,
        };

        let cpfp_flags_source =
            EagerVec::<ColumnarVec<PcoVec<TxIndex, StoredBool>, CpfpRoleId>>::forced_import(
                db,
                "cpfp_flags",
                version,
            )?;
        let flags = cpfp_flags_source.read_only_clone();

        Ok(Self {
            count,
            input_value: EagerVec::forced_import(db, "input_value", version)?,
            output_value: EagerVec::forced_import(db, "output_value", version)?,
            fee: PerTxDistribution::forced_import(db, "fee", v, indexes)?,
            fee_rate: EagerVec::forced_import(db, "fee_rate", v)?,
            effective_fee_rate: PerTxDistribution::forced_import(
                db,
                "effective_fee_rate",
                v,
                indexes,
            )?,
            cpfp_flags: CpfpFlags {
                is_cpfp_parent: flags.column("is_cpfp_parent", version, CpfpRoleId::Parent),
                is_cpfp_child: flags.column("is_cpfp_child", version, CpfpRoleId::Child),
            },
            cpfp_flags_source,
        })
    }
}

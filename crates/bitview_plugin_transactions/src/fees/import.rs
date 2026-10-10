use bitview_collections::Windows;
use bitview_vecs::{
    IndexSources, LazyWindowStartVec, PerBlockCumulativeRolling, PerTxDistribution,
};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, EagerVec, ImportableVec};

use super::{CountVecs, CpfpFlags, Vecs};

/// Bump this when fee/feerate aggregation logic changes (e.g., skip coinbase, skip zero-fee).
const VERSION: Version = Version::new(5);

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let v = version + VERSION;
        let count = |name| {
            PerBlockCumulativeRolling::import(
                db,
                name,
                version + Version::ONE,
                mappings,
                window_starts,
            )
        };
        let count = CountVecs {
            cpfp_parent: count("cpfp_parent_tx_count")?,
            cpfp_child: count("cpfp_child_tx_count")?,
        };

        Ok(Vecs {
            count,
            total: EagerVec::import(db, "fee_total", v)?,
            coinbase_value: EagerVec::import(db, "coinbase_value", v)?,
            fee: PerTxDistribution::import(db, "tx_fee", v, mappings)?,
            fee_rate: EagerVec::import(db, "fee_rate", v)?,
            effective_fee_rate: PerTxDistribution::import(db, "effective_fee_rate", v, mappings)?,
            cpfp_flags: CpfpFlags {
                is_cpfp_parent: EagerVec::import(db, "is_cpfp_parent", version + Version::ONE)?,
                is_cpfp_child: EagerVec::import(db, "is_cpfp_child", version + Version::ONE)?,
            },
        })
    }
}

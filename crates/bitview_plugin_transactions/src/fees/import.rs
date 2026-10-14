use bitview_collections::Windows;
use bitview_vecs::{
    IndexSources, LazyWindowStartVec, PerBlockCumulativeRolling, PerTxDistribution,
};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, EagerVec, ImportableVec};

use super::Vecs;
use crate::flagged::Flagged;

/// Bump this when fee/feerate aggregation logic changes (e.g., skip coinbase, skip zero-fee).
const VERSION: Version = Version::new(5);
/// Bump this when chunking changes (`brk_cpfp::linearize`): effective rates and CPFP roles.
const CHUNK_VERSION: Version = Version::TWO;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let v = version + VERSION;
        let cpfp_role = |flag, count| -> Result<_> {
            Ok(Flagged {
                flag: EagerVec::import(db, flag, version + CHUNK_VERSION)?,
                count: PerBlockCumulativeRolling::import(
                    db,
                    count,
                    version + CHUNK_VERSION,
                    mappings,
                    window_starts,
                )?,
            })
        };

        Ok(Vecs {
            total: EagerVec::import(db, "fee_total", v)?,
            total_cumulative: EagerVec::import(db, "fee_total_cumulative", v)?,
            coinbase_value: EagerVec::import(db, "coinbase_value", v)?,
            fee: PerTxDistribution::import(db, "tx_fee", v, mappings)?,
            fee_rate: EagerVec::import(db, "fee_rate", v)?,
            effective_fee_rate: PerTxDistribution::import(
                db,
                "effective_fee_rate",
                v + CHUNK_VERSION,
                mappings,
            )?,
            cpfp_parent: cpfp_role("is_cpfp_parent", "cpfp_parent_tx_count")?,
            cpfp_child: cpfp_role("is_cpfp_child", "cpfp_child_tx_count")?,
        })
    }
}

use bitview_primitives::Count;
use bitview_vecs::{IndexSources, LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, ImportableVec, PcoVec, ReadableCloneableVec};

use super::Vecs;
use crate::{distribution::PerTxDistribution, flagged::Classified};

/// Bump this when fee/feerate aggregation logic changes (e.g., skip coinbase, skip zero-fee).
const VERSION: Version = Version::new(5);
/// Bump this when chunking changes (`brk_cpfp::Cluster::linearize`): effective rates and CPFP roles.
const CHUNK_VERSION: Version = Version::TWO;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
        all_txs: &impl ReadableCloneableVec<Height, Count>,
    ) -> Result<Self> {
        let fee_version = version + VERSION;
        let cpfp_role = |flag, kind| {
            Classified::import(
                db,
                flag,
                kind,
                version + CHUNK_VERSION,
                mappings,
                window_starts,
                all_txs,
            )
        };

        Ok(Vecs {
            total: PcoVec::import(db, "fee_total", fee_version)?,
            total_cumulative: PcoVec::import(db, "fee_total_cumulative", fee_version)?,
            coinbase_value: PcoVec::import(db, "coinbase_value", fee_version)?,
            fee: PerTxDistribution::import(db, "tx_fee", fee_version, mappings)?,
            fee_rate: PcoVec::import(db, "tx_fee_rate", fee_version)?,
            effective_fee_rate: PerTxDistribution::import(
                db,
                "tx_effective_fee_rate",
                fee_version + CHUNK_VERSION,
                mappings,
            )?,
            cpfp_parent: cpfp_role("is_cpfp_parent", "cpfp_parent_tx")?,
            cpfp_child: cpfp_role("is_cpfp_child", "cpfp_child_tx")?,
        })
    }
}

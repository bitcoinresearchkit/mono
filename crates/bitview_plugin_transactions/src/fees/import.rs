use bitview_primitives::Count;
use bitview_vecs::{IndexSources, LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Height, Sats, Version};
use vecdb::{Database, ImportableVec, PcoVec, ReadableCloneableVec};

use super::{Vecs, ranges::Ranges};
use crate::{
    distribution::PerTxDistribution,
    flagged::{Classified, Totals},
};

/// Bump this when fee/feerate aggregation logic changes (e.g., skip coinbase, skip zero-fee).
const VERSION: Version = Version::new(5);
/// Bump this when chunking changes (`brk_cpfp::Cluster::linearize`, or the weights it measures
/// over): effective rates and CPFP roles.
const CHUNK_VERSION: Version = Version::new(5);

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
        totals: Totals<
            '_,
            impl ReadableCloneableVec<Height, Count>,
            impl ReadableCloneableVec<Height, Sats>,
        >,
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
                totals,
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
            ranges: Ranges::import(db, fee_version + CHUNK_VERSION, mappings, window_starts)?,
        })
    }
}

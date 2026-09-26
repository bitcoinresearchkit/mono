use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling, PerTxDistribution};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, EagerVec, ImportableVec};

use super::{CountVecs, CpfpFlags, Vecs};

/// Bump this when fee/feerate aggregation logic changes (e.g., skip coinbase, skip zero-fee).
const VERSION: Version = Version::new(5);

pub fn forced_import(
    db: &Database,
    version: Version,
    mappings: &MappingsVecs,
    window_starts: &Windows<&LazyWindowStartVec>,
) -> Result<Vecs> {
    let v = version + VERSION;
    let count = |name| {
        PerBlockCumulativeRolling::forced_import(
            db,
            name,
            version + Version::ONE,
            mappings,
            window_starts,
        )
    };
    let count = CountVecs {
        cpfp_parent: count("cpfp_parent_count")?,
        cpfp_child: count("cpfp_child_count")?,
    };

    Ok(Vecs {
        count,
        coinbase_value: EagerVec::forced_import(db, "coinbase_value", v)?,
        fee: PerTxDistribution::forced_import(db, "fee", v, mappings)?,
        fee_rate: EagerVec::forced_import(db, "fee_rate", v)?,
        effective_fee_rate: PerTxDistribution::forced_import(
            db,
            "effective_fee_rate",
            v,
            mappings,
        )?,
        cpfp_flags: CpfpFlags {
            is_cpfp_parent: EagerVec::forced_import(db, "is_cpfp_parent", version + Version::ONE)?,
            is_cpfp_child: EagerVec::forced_import(db, "is_cpfp_child", version + Version::ONE)?,
        },
    })
}

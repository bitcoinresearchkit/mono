use bitview_collections::Windows;
use bitview_vecs::{IndexSources, PerBlockCumulativeRolling, PercentPerBlock};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, ReadableCloneableVec};

use super::Vecs;

pub fn forced_import(
    db: &Database,
    version: Version,
    indexes: &IndexSources,
    window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
) -> Result<Vecs> {
    let version = version + Version::ONE;
    Ok(Vecs {
        count: PerBlockCumulativeRolling::forced_import(
            db,
            "tx_count_inscription",
            version,
            indexes,
            window_starts,
        )?,
        fees: PerBlockCumulativeRolling::forced_import(
            db,
            "inscription_fees",
            version,
            indexes,
            window_starts,
        )?,
        fee_share: PercentPerBlock::forced_import(db, "inscription_fee_share", version, indexes)?,
    })
}

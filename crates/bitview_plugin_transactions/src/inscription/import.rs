use bitview_collections::Windows;
use bitview_vecs::{IndexSources, PerBlockCumulativeRolling, PercentPerBlock};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, ReadableCloneableVec};

use super::Vecs;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        Ok(Vecs {
            count: PerBlockCumulativeRolling::import(
                db,
                "tx_count_inscription",
                version,
                indexes,
                window_starts,
            )?,
            fees: PerBlockCumulativeRolling::import(
                db,
                "inscription_fees",
                version,
                indexes,
                window_starts,
            )?,
            fee_share: PercentPerBlock::import(db, "inscription_fee_share", version, indexes)?,
        })
    }
}

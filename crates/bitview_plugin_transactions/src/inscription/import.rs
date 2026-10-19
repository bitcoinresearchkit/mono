use bitview_plugin_indexer::Indexer;
use bitview_primitives::PartsPerMillion32;
use bitview_vecs::{
    IndexSources, LazyPercentCumulativeRolling, PerBlockCumulativeRolling,
    ValuePerBlockCumulativeRolling, ops::Quotient, shapes::Windows,
};
use brk_error::Result;
use brk_types::{Height, Sats, Version};
use vecdb::{Database, ReadableCloneableVec};

use super::Vecs;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        chain_fees: &impl ReadableCloneableVec<Height, Sats>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let fees = ValuePerBlockCumulativeRolling::import(
            db,
            "inscription_fees",
            version + Version::ONE,
            indexes,
            window_starts,
        )?;
        let chain_share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
            Sats,
            Sats,
            Quotient<PartsPerMillion32>,
        >(
            "inscription_fee_chain_share",
            version,
            fees.cumulative.sats.resolutions.height_source(),
            chain_fees,
            window_starts,
            indexes,
        );
        Ok(Vecs {
            flag: indexer.vecs().transactions.features.inscription.flag_view(),
            count: PerBlockCumulativeRolling::import(
                db,
                "inscription_tx_count",
                version,
                indexes,
                window_starts,
            )?,
            fees,
            chain_share,
        })
    }
}

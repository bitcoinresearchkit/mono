use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PartsPerMillion32;
use bitview_transforms::{OneMinusPpm, Quotient};
use bitview_vecs::{
    LazyPercentCumulativeRolling, LazyWindowStartVec, ValuePerBlockCumulativeRolling,
    ValuePerBlockFull,
};
use brk_error::Result;
use brk_types::{Sats, Version};
use vecdb::Database;

use super::Vecs;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let coinbase = ValuePerBlockCumulativeRolling::import(
            db,
            "coinbase",
            version,
            mappings,
            window_starts,
        )?;
        let subsidy = ValuePerBlockCumulativeRolling::import(
            db,
            "subsidy",
            version,
            mappings,
            window_starts,
        )?;
        let fees = ValuePerBlockFull::import(db, "fees", version, mappings, window_starts)?;
        let fees_source = fees.cumulative_sats_source();

        let fee_share = LazyPercentCumulativeRolling::from_cumulative_ratio_with_numerator::<
            Sats,
            Sats,
            Quotient<PartsPerMillion32>,
        >(
            "fee_share",
            version,
            fees_source,
            coinbase.cumulative.sats.resolutions.height_source(),
            window_starts,
            mappings,
        );
        let subsidy_share = LazyPercentCumulativeRolling::from_lazy_source::<OneMinusPpm>(
            "subsidy_share",
            version,
            &fee_share,
        );
        Ok(Vecs {
            coinbase,
            subsidy,
            fees,
            unclaimed: ValuePerBlockCumulativeRolling::import(
                db,
                "unclaimed_rewards",
                version,
                mappings,
                window_starts,
            )?,
            fee_share,
            subsidy_share,
        })
    }
}

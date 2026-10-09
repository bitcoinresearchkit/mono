use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{PartsPerMillion32, PartsPerMillion64};
use bitview_transforms::{OneMinusPpm, Quotient};
use bitview_vecs::{
    LazyPercentCumulativeRolling, LazyRatioRollingWindows, LazyWindowStartVec,
    ValuePerBlockCumulative, ValuePerBlockCumulativeRolling, ValuePerBlockFull,
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
        let fee_to_subsidy = LazyRatioRollingWindows::from_cumulative_ratio_with_numerator::<
            Sats,
            Sats,
            Quotient<PartsPerMillion64>,
        >(
            "fee_to_subsidy",
            version + Version::ONE,
            fees_source,
            subsidy.cumulative.sats.resolutions.height_source(),
            window_starts,
            mappings,
        );

        Ok(Vecs {
            coinbase,
            subsidy,
            fees,
            output_volume: ValuePerBlockCumulativeRolling::import(
                db,
                "output_volume",
                version + Version::ONE,
                mappings,
                window_starts,
            )?,
            unclaimed: ValuePerBlockCumulative::import(db, "unclaimed_rewards", version, mappings)?,
            fee_share,
            subsidy_share,
            fee_to_subsidy,
        })
    }
}

use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PartsPerMillion32;
use bitview_vecs::{
    LazyPercentCumulativeRolling, LazyWindowStartVec, ValuePerBlockCumulativeRolling,
    ValuePerBlockFull,
    ops::{OneMinusPpm, Quotient},
    shapes::Windows,
};
use brk_error::Result;
use brk_types::{Sats, Version};
use vecdb::{Database, ReverseOperands};

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

        // Claimed subsidy over coinbase, so an underclaimed coinbase stays within 0..=100%;
        // the coinbase is read first.
        let subsidy_share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
            Sats,
            Sats,
            ReverseOperands<Quotient<PartsPerMillion32>>,
        >(
            "subsidy_share",
            version,
            coinbase.cumulative.sats.resolutions.height_source(),
            subsidy.cumulative.sats.resolutions.height_source(),
            window_starts,
            mappings,
        );
        let fee_share = LazyPercentCumulativeRolling::from_lazy_source::<OneMinusPpm>(
            "fee_share",
            version,
            &subsidy_share,
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

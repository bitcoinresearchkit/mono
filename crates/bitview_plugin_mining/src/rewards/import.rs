use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Multiple, PartsPerMillion32};
use bitview_vecs::{
    LazyIndexedVec, LazyPerBlock, LazyPercentCumulativeRolling, LazyWindowStartVec,
    ValuePerBlockCumulativeRolling, ValuePerBlockFull,
    ops::{OneMinusPpm, Quotient},
    shapes::Windows,
};
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::{Database, Ident, ReverseOperands};

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
        let puell_source = LazyIndexedVec::new(
            "puell_multiple_source",
            version,
            &subsidy.sum._24h.cents.height,
            &subsidy.sum._1y.cents.height,
            |_, day: Cents, year: Cents| Multiple::from(365.0 * f64::from(day) / f64::from(year)),
        );
        let puell_multiple = LazyPerBlock::from_height_source::<Ident>(
            "puell_multiple",
            version,
            &puell_source,
            mappings,
        );

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
            puell_multiple,
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

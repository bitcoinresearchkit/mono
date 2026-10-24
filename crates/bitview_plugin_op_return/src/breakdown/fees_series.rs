use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPercentCumulativeRolling, LazyWindowStartVec, ValuePerBlockCumulativeRolling,
    ops::Quotient, shapes::Windows,
};
use brk_error::Result;
use brk_types::{Height, Sats, Version};
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct FeesSeries<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub fees: ValuePerBlockCumulativeRolling<M>,
    /// These fees divided by all transaction fees in the chain over the same
    /// cumulative or trailing window; undefined (null) when those fees are zero.
    pub chain_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

impl FeesSeries {
    pub fn import(
        db: &Database,
        prefix: &str,
        version: Version,
        chain_fees: &impl ReadableCloneableVec<Height, Sats>,
        window_starts: &Windows<&LazyWindowStartVec>,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let fees = ValuePerBlockCumulativeRolling::import(
            db,
            &format!("{prefix}_fees"),
            version,
            mappings,
            window_starts,
        )?;
        let chain_share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
            Sats,
            Sats,
            Quotient<PartsPerMillion32>,
        >(
            &format!("{prefix}_fee_chain_share"),
            version,
            fees.cumulative.sats.resolutions.height_source(),
            chain_fees,
            window_starts,
            mappings,
        );

        Ok(Self { fees, chain_share })
    }
}

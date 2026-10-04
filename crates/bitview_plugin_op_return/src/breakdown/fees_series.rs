use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PartsPerMillion32;
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFixedRatioCumulativeRolling, LazyWindowStartVec, PerBlockCumulativeRolling,
};
use brk_types::{Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{ReadableCloneableVec, Rw, StorageMode};

#[derive(Deref, DerefMut, Traversable)]
pub struct FeesSeries<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub fees: PerBlockCumulativeRolling<Sats, M>,
    /// Fees of transactions in a breakdown bucket divided by all transaction
    /// fees over the same cumulative or trailing window.
    pub fee_share: LazyFixedRatioCumulativeRolling<PartsPerMillion32>,
}

impl FeesSeries {
    pub fn new(
        prefix: &str,
        version: Version,
        fees: PerBlockCumulativeRolling<Sats>,
        chain_fees: &impl ReadableCloneableVec<Height, Sats>,
        window_starts: &Windows<&LazyWindowStartVec>,
        mappings: &MappingsVecs,
    ) -> Self {
        let fee_share = LazyFixedRatioCumulativeRolling::from_cumulative_ratio::<
            Sats,
            Sats,
            Quotient<PartsPerMillion32>,
        >(
            &format!("{prefix}_fee_share"),
            version,
            fees.cumulative.resolutions.height_source(),
            chain_fees,
            window_starts,
            mappings,
        );

        Self { fees, fee_share }
    }
}

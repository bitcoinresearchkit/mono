use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PartsPerMillion32;
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPercentCumulativeRolling, LazyWindowStartVec, ValuePerBlockCumulativeRolling,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{
    AnyStoredVec, AnyVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode, WritableVec,
};

#[derive(Traversable)]
pub struct FeesSeries<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub fees: ValuePerBlockCumulativeRolling<M>,
    /// These fees divided by all transaction fees in the chain over the same
    /// cumulative or trailing window.
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

    pub fn len(&self) -> usize {
        self.fees.cumulative.sats.height.len()
    }

    pub fn push_block(&mut self, value: Sats) {
        let cumulative = &mut self.fees.cumulative.sats.height;
        let last = cumulative.collect_last().unwrap_or_default();
        cumulative.push(last + value);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.fees.cumulative.sats.height
    }

    pub fn compute_cents(
        &mut self,
        max_from: Height,
        price_cents: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.fees.compute_cents(max_from, price_cents, exit)
    }
}

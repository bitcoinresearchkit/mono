use bitview_cohort::{ByType, SpendableType};
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CumulativeSource, IndexSources, LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling,
    ValuePerBlockCumulativeRolling, shapes::Windows,
};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub(crate) types: ByType<OutputTypeVecs>,
    #[traversable(hidden)]
    pub(crate) count_stored: ByType<CumulativeSource<Count, M>>,
    #[traversable(hidden)]
    pub(crate) tx_count_stored: ByType<CumulativeSource<Count, M>>,
    #[traversable(flatten)]
    pub(crate) values: SpendableType<OutputValue<M>>,
}

/// What a spendable type's outputs were sent.
#[derive(Traversable)]
pub(crate) struct OutputValue<M: StorageMode = Rw> {
    /// All outputs of the type ever created, in bitcoin.
    #[traversable(lead = "Value sent to outputs of this type.")]
    pub(crate) value: ValuePerBlockCumulativeRolling<M>,
}

impl OutputValue {
    pub(crate) fn import_all(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<SpendableType<Self>> {
        SpendableType::try_from_fn(|kind| {
            let name = format!("{}_output_value", kind.key());
            Ok(Self {
                value: ValuePerBlockCumulativeRolling::import(
                    db,
                    &name,
                    version,
                    indexes,
                    window_starts,
                )?,
            })
        })
    }
}

#[derive(Clone, Traversable)]
pub struct OutputTypeVecs {
    /// Number of outputs of the type.
    #[traversable(lead = "Outputs of this type.")]
    pub(crate) count: LazyPerBlockCumulativeRolling<Count>,
    /// Outputs of the type divided by all outputs over the same cumulative or
    /// trailing window.
    #[traversable(lead = "This type's share of outputs.")]
    pub(crate) share: LazyPercentCumulativeRolling<PartsPerMillion32>,
    /// Number of transactions with at least one output of the type.
    #[traversable(lead = "Transactions creating this type.")]
    pub(crate) tx_count: LazyPerBlockCumulativeRolling<Count>,
    /// Transactions with at least one output of the type divided by all
    /// transactions over the same cumulative or trailing window.
    #[traversable(lead = "Transactions creating this type, as a share of all transactions.")]
    pub(crate) tx_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

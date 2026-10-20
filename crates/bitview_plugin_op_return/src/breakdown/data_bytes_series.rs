use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Bytes, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPercentCumulativeRolling, LazyWindowStartVec, PerBlockCumulativeRolling, ops::Quotient,
    shapes::Windows,
};
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{ReadableCloneableVec, Rw, StorageMode};

#[derive(Deref, DerefMut, Traversable)]
pub struct DataBytesSeries<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub data_bytes: PerBlockCumulativeRolling<Bytes, M>,
    /// These data bytes divided by all serialized block bytes in the chain over the same
    /// cumulative or trailing window.
    pub chain_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

impl DataBytesSeries {
    pub fn new(
        prefix: &str,
        version: Version,
        data_bytes: PerBlockCumulativeRolling<Bytes>,
        block_size: &impl ReadableCloneableVec<Height, Bytes>,
        window_starts: &Windows<&LazyWindowStartVec>,
        mappings: &MappingsVecs,
    ) -> Self {
        let chain_share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
            Bytes,
            Bytes,
            Quotient<PartsPerMillion32>,
        >(
            &format!("{prefix}_data_chain_share"),
            version,
            data_bytes.cumulative.resolutions.height_source(),
            block_size,
            window_starts,
            mappings,
        );

        Self {
            data_bytes,
            chain_share,
        }
    }
}

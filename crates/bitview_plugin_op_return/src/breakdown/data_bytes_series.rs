use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Bytes, PartsPerMillion32};
use bitview_transforms::RatioBytes;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPercentPerBlock, PerBlockCumulativeRolling};
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{ReadableCloneableVec, Rw, StorageMode};

#[derive(Deref, DerefMut, Traversable)]
pub struct DataBytesSeries<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub data_bytes: PerBlockCumulativeRolling<Bytes, M>,
    /// Cumulative `OP_RETURN` data bytes in a breakdown bucket divided by
    /// cumulative data bytes across all `OP_RETURN` outputs.
    pub data_share: LazyPercentPerBlock<PartsPerMillion32>,
    /// Cumulative `OP_RETURN` data bytes in a breakdown bucket divided by
    /// cumulative serialized block bytes.
    pub chain_share: LazyPercentPerBlock<PartsPerMillion32>,
}

impl DataBytesSeries {
    pub fn new(
        prefix: &str,
        version: Version,
        data_bytes: PerBlockCumulativeRolling<Bytes>,
        total_data: &impl ReadableCloneableVec<Height, Bytes>,
        block_size: &impl ReadableCloneableVec<Height, Bytes>,
        mappings: &MappingsVecs,
    ) -> Self {
        let data_share = LazyPercentPerBlock::from_ratio::<Bytes, _, RatioBytes<PartsPerMillion32>>(
            &format!("{prefix}_data_share"),
            version,
            data_bytes.cumulative.resolutions.height_source(),
            total_data,
            mappings,
        );
        let chain_share = LazyPercentPerBlock::from_ratio::<Bytes, _, RatioBytes<PartsPerMillion32>>(
            &format!("{prefix}_chain_share"),
            version,
            data_bytes.cumulative.resolutions.height_source(),
            block_size,
            mappings,
        );

        Self {
            data_bytes,
            data_share,
            chain_share,
        }
    }
}

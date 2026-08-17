use brk_traversable::Traversable;
use brk_types::{Bytes, Height, PartsPerMillion32, StoredU64, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{CachedBoxedVec, ColumnId};

use crate::{
    indexes,
    internal::{LazyColumnPerBlockCumulativeRolling, LazyPercentPerBlock, RatioBytes},
};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct DataBytesSeries<C: ColumnId> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub data_bytes: LazyColumnPerBlockCumulativeRolling<Bytes, C>,
    /// Cumulative `OP_RETURN` data bytes in the selected bucket divided by
    /// cumulative data bytes across all `OP_RETURN` outputs.
    pub data_share: LazyPercentPerBlock<PartsPerMillion32>,
    /// Cumulative `OP_RETURN` data bytes in the selected bucket divided by
    /// cumulative serialized block bytes.
    pub chain_share: LazyPercentPerBlock<PartsPerMillion32>,
}

impl<C: ColumnId> DataBytesSeries<C> {
    pub(super) fn new(
        prefix: &str,
        version: Version,
        data_bytes: LazyColumnPerBlockCumulativeRolling<Bytes, C>,
        total_data: CachedBoxedVec<Height, Bytes>,
        block_size: CachedBoxedVec<Height, StoredU64>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let data_share =
            LazyPercentPerBlock::from_cached_ratio::<Bytes, _, RatioBytes<PartsPerMillion32>>(
                &format!("{prefix}_data_share"),
                version,
                &data_bytes.cumulative.height,
                total_data,
                indexes,
            );
        let chain_share =
            LazyPercentPerBlock::from_cached_ratio::<Bytes, _, RatioBytes<PartsPerMillion32>>(
                &format!("{prefix}_chain_share"),
                version,
                &data_bytes.cumulative.height,
                block_size,
                indexes,
            );

        Self {
            data_bytes,
            data_share,
            chain_share,
        }
    }
}

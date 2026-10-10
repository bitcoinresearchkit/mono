use bitview_traversable::Traversable;
use bitview_vecs::LazySpotValuePerBlock;

/// One side of an age range's mobility split.
#[derive(Clone, Traversable)]
pub struct SideVecs {
    /// Supply in the age range times that weight, rounded down to whole
    /// satoshis.
    pub supply: LazySpotValuePerBlock,
}

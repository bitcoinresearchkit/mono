use bitview_traversable::Traversable;
use bitview_vecs::LazySpotValuePerBlock;

/// One side of an age range's wakefulness split.
#[derive(Clone, Traversable)]
pub struct SideVecs {
    /// Supply in the age range: the awake side is the supply times the wakefulness, rounded half
    /// up to the nearest satoshi, and the dormant side holds the rest. While the wakefulness is
    /// undefined, the dormant side holds the range's whole supply.
    pub supply: LazySpotValuePerBlock,
}

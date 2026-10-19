use bitview_traversable::Traversable;
use bitview_vecs::LazySpotValuePerBlock;

/// One side of an age range's mobility split.
#[derive(Clone, Traversable)]
pub struct SideVecs {
    /// Supply in the age range: the mobile side is the supply times the mobility, rounded half
    /// up to the nearest satoshi, and the immobile side holds the rest.
    pub supply: LazySpotValuePerBlock,
}

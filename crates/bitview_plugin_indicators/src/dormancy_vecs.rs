use bitview_primitives::Float32;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct DormancyVecs {
    /// Trailing 24-hour dormancy divided by all-chain supply in BTC at the
    /// represented block. Dormancy is trailing 24-hour coin days destroyed
    /// divided by trailing 24-hour transfer volume in BTC. Returns zero when
    /// supply is zero. Larger values mean older coins were spent relative to
    /// the size of the supply.
    pub supply_adj: LazyPerBlock<Float32>,
    /// All-chain supply in BTC at the represented block divided by trailing
    /// 24-hour dormancy. Dormancy is trailing 24-hour coin days destroyed
    /// divided by trailing 24-hour transfer volume in BTC. Returns zero when
    /// dormancy is zero. Larger values correspond to younger average spent
    /// coins relative to the size of the supply.
    pub flow: LazyPerBlock<Float32>,
}

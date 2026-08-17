use brk_traversable::Traversable;
use brk_types::StoredF32;

use crate::internal::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct DormancyVecs {
    /// Trailing 24-hour dormancy divided by the current all-chain supply in BTC.
    /// Dormancy is trailing 24-hour coin days destroyed divided by trailing
    /// 24-hour transfer volume in BTC. Returns zero when supply is zero.
    pub supply_adj: LazyPerBlock<StoredF32>,
    /// Current all-chain supply in BTC divided by trailing 24-hour dormancy.
    /// Dormancy is trailing 24-hour coin days destroyed divided by trailing
    /// 24-hour transfer volume in BTC. Returns zero when dormancy is zero.
    pub flow: LazyPerBlock<StoredF32>,
}

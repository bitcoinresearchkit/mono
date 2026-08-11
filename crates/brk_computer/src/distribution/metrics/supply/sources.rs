use crate::internal::LazySpotValuePerBlock;

#[derive(Clone)]
pub struct SupplySources {
    pub total: LazySpotValuePerBlock,
    pub in_profit: LazySpotValuePerBlock,
}

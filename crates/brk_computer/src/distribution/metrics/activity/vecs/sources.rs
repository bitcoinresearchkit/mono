use crate::internal::LazyValuePerBlockCumulativeRolling;

#[derive(Clone)]
pub struct ActivitySources {
    pub transfer_volume: LazyValuePerBlockCumulativeRolling,
}

use bitview_compute::LazyValuePerBlockCumulativeRolling;

#[derive(Clone)]
pub struct ActivitySources {
    pub transfer_volume: LazyValuePerBlockCumulativeRolling,
}

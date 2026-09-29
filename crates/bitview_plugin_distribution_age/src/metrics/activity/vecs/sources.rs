use bitview_vecs::LazyValuePerBlockCumulativeRolling;

#[derive(Clone)]
pub struct ActivitySources {
    pub transfer_volume: LazyValuePerBlockCumulativeRolling,
}

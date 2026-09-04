use brk_types::{PoolSlug, TimePeriod};

use super::TipJsonCache;

#[derive(Clone, Default)]
pub(crate) struct MiningCaches {
    pub(crate) pool_stats: TipJsonCache<TimePeriod>,
    pub(crate) pools_hashrate: TipJsonCache<Option<TimePeriod>>,
    pub(crate) pool_hashrate: TipJsonCache<PoolSlug>,
    pub(crate) hashrate: TipJsonCache<Option<TimePeriod>>,
    pub(crate) difficulty_adjustments: TipJsonCache<Option<TimePeriod>>,
    pub(crate) block_fees: TipJsonCache<TimePeriod>,
    pub(crate) block_fee_rates: TipJsonCache<TimePeriod>,
    pub(crate) block_sizes_weights: TipJsonCache<TimePeriod>,
    pub(crate) block_rewards: TipJsonCache<TimePeriod>,
}

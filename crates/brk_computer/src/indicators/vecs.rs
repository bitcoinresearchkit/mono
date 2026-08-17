use brk_plugin::{Plugin, PluginGate};
use brk_traversable::Traversable;
use brk_types::{PartsPerMillion32, PartsPerMillion64, StoredF32};
use vecdb::{Database, Rw, StorageMode};

use super::dormancy_vecs::DormancyVecs;
use crate::internal::{LazyPerBlock, LazyRatioPerBlock, PerBlock, PercentPerBlock, RatioPerBlock};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub(crate) db: Database,
    pub puell_multiple: RatioPerBlock<PartsPerMillion64, M>,
    pub nvt: LazyRatioPerBlock<PartsPerMillion64>,
    pub gini: PercentPerBlock<PartsPerMillion32, M>,
    pub rhodl_ratio: RatioPerBlock<PartsPerMillion64, M>,
    pub thermo_cap_multiple: LazyRatioPerBlock<PartsPerMillion64>,
    /// Trailing 24-hour coin days destroyed divided by the current all-chain
    /// supply in BTC. Returns zero when supply is zero.
    pub coindays_destroyed_supply_adj: LazyPerBlock<StoredF32>,
    /// Trailing 365-day total of coin days destroyed divided by the current
    /// all-chain supply in BTC. Despite the series name, the numerator is not
    /// divided by 365. Returns zero when supply is zero.
    pub coinyears_destroyed_supply_adj: LazyPerBlock<StoredF32>,
    pub dormancy: DormancyVecs,
    /// Current all-chain supply in satoshis divided by the current block's
    /// derived subsidy component annualized at 52,560 blocks. Returns zero when
    /// the annualized flow is zero.
    pub stock_to_flow: LazyPerBlock<StoredF32>,
    /// Current all-chain supply-in-profit share multiplied by the population
    /// standard deviation of per-block trailing-24-hour spot-price returns over
    /// the trailing 30-day monotonic-time window, scaled by the square root of
    /// 30. Returns zero when total supply is zero.
    pub seller_exhaustion: PerBlock<StoredF32, M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> &'static str {
        super::DB_NAME
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

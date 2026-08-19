use bitview_plugin::{Plugin, PluginGate, PluginId};
use bitview_traversable::Traversable;
use brk_types::{PartsPerMillion32, PartsPerMillion64, StoredF32};
use vecdb::{Database, Rw, StorageMode};

use super::dormancy_vecs::DormancyVecs;
use bitview_compute::{LazyPerBlock, LazyRatioPerBlock, PerBlock, PercentPerBlock, RatioPerBlock};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,
    /// Current block's derived subsidy value in USD divided by its trailing
    /// 365-day per-block arithmetic mean.
    pub puell_multiple: RatioPerBlock<PartsPerMillion64, M>,
    /// Network Value to Transactions ratio: market capitalization divided by
    /// trailing-24-hour transfer volume, both valued in USD. Returns zero when
    /// the ratio is not finite.
    pub nvt: LazyRatioPerBlock<PartsPerMillion64>,
    /// Approximate Gini coefficient of UTXO-held supply, derived from the
    /// Lorenz curve across ordered UTXO-amount cohorts. Zero means equal
    /// distribution; larger values mean greater concentration.
    pub gini: PercentPerBlock<PartsPerMillion32, M>,
    /// Realized HODL ratio: realized capitalization of 1-day-to-1-week-old
    /// UTXOs divided by that of 1-to-2-year-old UTXOs. Returns zero when the
    /// ratio is not finite.
    pub rhodl_ratio: RatioPerBlock<PartsPerMillion64, M>,
    /// Market capitalization divided by thermo capitalization, the cumulative
    /// USD value of derived block subsidies. Returns zero when the ratio is not
    /// finite.
    pub thermo_cap_multiple: LazyRatioPerBlock<PartsPerMillion64>,
    /// Trailing 24-hour coin days destroyed divided by the current all-chain
    /// supply in BTC. Returns zero when supply is zero.
    pub coindays_destroyed_supply_adj: LazyPerBlock<StoredF32>,
    /// Trailing 365-day coin years destroyed divided by the current all-chain
    /// supply in BTC. Returns zero when supply is zero.
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
    Self: Traversable + Send + Sync,
{
    fn id(&self) -> PluginId {
        crate::ID
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
mod compute;
mod import;

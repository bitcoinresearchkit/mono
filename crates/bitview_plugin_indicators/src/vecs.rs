use bitview_plugin::{Plugin, PluginGate, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::{PartsPerMillion32, PartsPerMillion64, StoredF32};
use vecdb::{Database, Rw, StorageMode};

use super::dormancy_vecs::DormancyVecs;
use crate::STORAGE;
use bitview_compute::{LazyPerBlock, LazyRatioPerBlock, PerBlock, PercentPerBlock, RatioPerBlock};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,
    /// Puell Multiple: the represented block's derived subsidy value in USD
    /// divided by its trailing 365-day per-block arithmetic mean. Values above
    /// one mean current subsidy revenue is above its one-year average; values
    /// below one mean it is below average.
    pub puell_multiple: RatioPerBlock<PartsPerMillion64, M>,
    /// Network Value to Transactions ratio: market capitalization divided by
    /// trailing-24-hour transfer volume, both valued in USD. Returns zero when
    /// the ratio is not finite. Larger values mean the network's market value
    /// is high relative to the on-chain value transferred in the last day.
    pub nvt: LazyRatioPerBlock<PartsPerMillion64>,
    /// Approximate Gini coefficient of UTXO-held supply, derived from the
    /// Lorenz curve across ordered UTXO-amount cohorts. Zero means equal
    /// distribution; larger values mean greater concentration.
    pub gini: PercentPerBlock<PartsPerMillion32, M>,
    /// Realized HODL ratio: realized capitalization of 1-day-to-1-week-old
    /// UTXOs divided by that of 1-to-2-year-old UTXOs. Returns zero when the
    /// ratio is not finite. Larger values mean more creation-date capital sits
    /// in the young cohort relative to the older cohort.
    pub rhodl_ratio: RatioPerBlock<PartsPerMillion64, M>,
    /// Market capitalization divided by thermo capitalization, the cumulative
    /// USD value of derived block subsidies. Returns zero when the ratio is not
    /// finite. Larger values mean Bitcoin's market value is higher relative to
    /// the historical issuance value assigned to miners.
    pub thermo_cap_multiple: LazyRatioPerBlock<PartsPerMillion64>,
    /// Trailing 24-hour coin days destroyed divided by all-chain supply in BTC
    /// at the represented block. Larger values mean more accumulated holding
    /// time was consumed by spending relative to the supply. Returns zero when
    /// supply is zero.
    pub coindays_destroyed_supply_adj: LazyPerBlock<StoredF32>,
    /// Trailing 365-day coin years destroyed divided by all-chain supply in BTC
    /// at the represented block. Larger values mean more accumulated holding
    /// time was consumed by spending relative to the supply. Returns zero when
    /// supply is zero.
    pub coinyears_destroyed_supply_adj: LazyPerBlock<StoredF32>,
    pub dormancy: DormancyVecs,
    /// All-chain supply in satoshis at the represented block divided by that
    /// block's derived subsidy component annualized at 52,560 blocks. Returns
    /// zero when the annualized flow is zero. The value approximates how many
    /// years of subsidy issuance at the represented block's rate would equal
    /// the current supply.
    pub stock_to_flow: LazyPerBlock<StoredF32>,
    /// Seller Exhaustion Constant: all-chain supply-in-profit share at the
    /// represented block multiplied by the population standard deviation of
    /// per-block trailing-24-hour spot-price returns over the trailing 30-day
    /// monotonic-time window, scaled by the square root of 30. Low values occur
    /// when little supply is in profit, recent volatility is low, or both;
    /// high values require both a profitable supply and volatile price. Returns
    /// zero when total supply is zero.
    pub seller_exhaustion: PerBlock<StoredF32, M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
mod compute;
mod import;

mod compute;
mod dependencies;
mod import;
mod velocity;

pub use dependencies::Dependencies;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{Multiple, PartsPerMillionSigned64};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlock, LazyPerBlock, LazyPercentPerBlock, LazyRollingDeltasFiatFromHeight,
    ValuePerBlockCumulativeRolling,
};
use brk_types::{Bitcoin, Cents, CentsSigned, Version};
use vecdb::{Database, Rw, StorageMode};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("supply"), Version::new(10));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Total value of all unspent transaction outputs in the UTXO set; its USD
    /// value is the market cap.
    #[traversable(lead = "Circulating supply: all the bitcoin in existence, unspent.")]
    circulating: LazyPerBlock<Bitcoin, Bitcoin>,
    /// Value made provably unspendable: the genesis subsidy, the values of outputs
    /// Bitcoin Core leaves out of its UTXO set (`OP_RETURN`, or a locking script over
    /// 10,000 bytes), unclaimed block rewards and the two coinbase outputs that BIP30
    /// duplicates overwrote (blocks 91812 and 91722, at 91842 and 91880).
    #[traversable(lead = "Burned: bitcoin made unspendable for good.")]
    burned: ValuePerBlockCumulativeRolling<M>,
    /// Scheduled annual issuance over the circulating supply: the represented
    /// block's scheduled subsidy times 52,560 blocks, divided by the supply. Null
    /// without supply (block 0, whose output is unspendable).
    #[traversable(lead = "Inflation rate: the new bitcoin a year brings, against the supply.")]
    pub inflation_rate: LazyPercentPerBlock<PartsPerMillionSigned64>,
    velocity: velocity::Vecs,
    /// Circulating supply valued at the represented block's Bitcoin spot price.
    #[traversable(wrap = "market_cap", rename = "value")]
    #[traversable(lead = "Market cap: all the bitcoin, at the price of the day.")]
    market_cap: LazyFiatPerBlock<Cents>,
    /// Absolute and relative change in market capitalization from the last
    /// block before each supported trailing monotonic-time window.
    #[traversable(wrap = "market_cap", rename = "delta")]
    #[traversable(lead = "Market cap change: how much the market cap changed.")]
    market_cap_delta: LazyRollingDeltasFiatFromHeight<Cents, CentsSigned, PartsPerMillionSigned64>,
    /// Network value to transactions (NVT): market cap divided by trailing-24h
    /// transfer volume in USD. Volume is non-coinbase transaction input value,
    /// valued at each spending block's price; it includes change and self-transfers,
    /// so this is unadjusted on-chain volume, not economic or entity-adjusted volume.
    /// Null without a wholly priced volume window or when volume is zero.
    #[traversable(lead = "NVT: the network's market value against a day's unadjusted transfer value.")]
    nvt: LazyPerBlock<Multiple>,
    /// NVT signal (Dmitry Kalichkin, 2018): market cap divided by the mean daily transfer volume
    /// in USD over the trailing 90 days (the 90-day sum over 90), the volume `nvt` uses. The
    /// window ends at the represented block, so each value reads only what came before it. Null
    /// without a wholly priced 90-day window or when volume is zero.
    #[traversable(lead = "NVT signal: the market cap against 90 days of transfer volume, Kalichkin's smoothed NVT.")]
    nvt_signal: LazyPerBlock<Multiple>,
}

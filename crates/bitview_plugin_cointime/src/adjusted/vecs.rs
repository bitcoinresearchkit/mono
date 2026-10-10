use bitview_primitives::{PartsPerMillionSigned32, Ratio64, Years};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, PerBlock, PercentPerBlock};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Cointime-adjusted supply inflation rate: the scheduled annual issuance
    /// over the circulating supply, multiplied by `liveliness / (1 - liveliness)`,
    /// the ratio of active to vaulted supply. Liveliness is cumulative coinblocks
    /// destroyed divided by cumulative coinblocks created. Null while the supply
    /// is at most 50 BTC. Higher values combine faster issuance with a larger
    /// active-to-vaulted holding-time ratio.
    pub inflation_rate: PercentPerBlock<PartsPerMillionSigned32, M>,
    /// Cointime-adjusted stock-to-flow: one over the adjusted inflation rate,
    /// the years of adjusted issuance that would equal the supply. Null when the
    /// adjusted inflation rate is zero or negative.
    pub stock_to_flow: LazyPerBlock<Years, PartsPerMillionSigned32>,
    /// Cointime-adjusted transaction velocity: velocity divided by liveliness,
    /// the turnover of the active supply rather than of all supply. Null until
    /// the first coins are spent (liveliness zero).
    pub velocity: Velocity<M>,
}

#[derive(Traversable)]
pub struct Velocity<M: StorageMode = Rw> {
    /// Trailing 365-day transfer volume in BTC over the active supply.
    pub btc: PerBlock<Ratio64, M>,
    /// Trailing 365-day transfer volume in USD over the active capitalization.
    pub usd: PerBlock<Ratio64, M>,
}

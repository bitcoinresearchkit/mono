use bitview_primitives::{PartsPerMillionSigned32, Ratio64};
use bitview_traversable::Traversable;
use bitview_vecs::{PerBlock, PercentPerBlock};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Cointime-adjusted supply inflation rate: the scheduled annual issuance
    /// over the circulating supply, multiplied by `liveliness / (1 - liveliness)`,
    /// the ratio of active to vaulted supply. Null without supply (block 0).
    pub inflation_rate: PercentPerBlock<PartsPerMillionSigned32, M>,
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

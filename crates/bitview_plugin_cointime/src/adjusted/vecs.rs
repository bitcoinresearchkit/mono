use bitview_primitives::{PartsPerMillionSigned32, StoredF64};
use bitview_traversable::Traversable;
use bitview_vecs::{PerBlock, PercentPerBlock};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Cointime-adjusted supply inflation rate: trailing 365-day circulating supply growth divided by starting supply, multiplied by `liveliness / (1 - liveliness)`. Liveliness is cumulative coinblocks destroyed divided by cumulative coinblocks created. Returns NaN while starting supply is at most 50 BTC. Higher values combine faster supply growth with a larger active-to-vaulted holding-time ratio.
    pub inflation_rate: PercentPerBlock<PartsPerMillionSigned32, M>,
    /// Cointime-adjusted native transaction velocity: trailing 365-day transfer
    /// volume in satoshis divided by all-chain supply at the represented block,
    /// multiplied by `liveliness / (1 - liveliness)`. Liveliness is cumulative
    /// coinblocks destroyed divided by cumulative coinblocks created. Higher
    /// values mean more native-unit turnover after emphasizing consumed over
    /// still-stored holding time.
    pub tx_velocity_native: PerBlock<StoredF64, M>,
    /// Cointime-adjusted fiat transaction velocity: trailing 365-day transfer
    /// volume in cents divided by all-chain market capitalization at the
    /// represented block, multiplied by `liveliness / (1 - liveliness)`.
    /// Liveliness is cumulative coinblocks destroyed divided by cumulative
    /// coinblocks created. Higher values mean more USD-value turnover after
    /// emphasizing consumed over still-stored holding time.
    pub tx_velocity_fiat: PerBlock<StoredF64, M>,
}

use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

mod hash_price_value;
mod hash_rate_sma;
mod rate;

pub use hash_price_value::HashPriceValueVecs;
pub use hash_rate_sma::HashRateSmaVecs;
pub use rate::RateVecs;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub rate: RateVecs<M>,
    /// Estimated miner revenue over the trailing 24-hour window, with each
    /// coinbase output valued in USD at its block's spot price, divided by the
    /// current estimated network hash rate.
    pub price: HashPriceValueVecs<M>,
    /// Coinbase output value in satoshis over the trailing 24-hour window,
    /// divided by the current estimated network hash rate.
    pub value: HashPriceValueVecs<M>,
}

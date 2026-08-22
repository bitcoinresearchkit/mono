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
    /// Network hash-rate estimates derived from block production. The base
    /// estimate multiplies the represented block's difficulty-implied target
    /// hash rate by the number of blocks in the trailing 24 hours and divides by
    /// the 144 blocks expected at Bitcoin's ten-minute target; the represented
    /// block's difficulty is used for the whole estimate.
    pub rate: RateVecs<M>,
    /// Estimated miner revenue over the trailing 24-hour window, with each
    /// coinbase output valued in USD at its block's spot price, divided by the
    /// represented block's estimated network hash rate. It estimates daily USD
    /// revenue per unit of mining hash rate, before costs.
    pub price: HashPriceValueVecs<M>,
    /// Coinbase output value in satoshis over the trailing 24-hour window,
    /// divided by the represented block's estimated network hash rate. It
    /// estimates daily bitcoin revenue per unit of mining hash rate, before
    /// costs.
    pub value: HashPriceValueVecs<M>,
}

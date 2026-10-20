use brk_types::{Bitcoin, Dollars};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Columnar buckets in ascending price order: entry `i` of each column is bucket `i`.
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
pub struct UrpdBuckets {
    /// Each bucket's price, in USD: its lower bound when aggregated, and for `raw` the price its
    /// coins' creation prices round to (four significant digits of the price in cents).
    pub price: Vec<Dollars>,
    /// Supply held with a last-move price inside each bucket, in BTC. With a `wakefulness` or
    /// `mobility` weight, the weighted supply, floored to whole satoshis at each grid price.
    pub supply: Vec<Bitcoin>,
    /// Realized cap contribution in USD: `price * supply` summed over the coins in each bucket,
    /// each coin's creation price taken on the price grid (the `raw` bucket price, within 0.05%
    /// of the exact price), rounded to the cent: a bucket worth under half a cent shows 0. The
    /// buckets therefore need not sum to the cohort's realized cap series.
    pub realized_cap: Vec<Dollars>,
    /// Unrealized P&L in USD at the represented block: `close * supply` minus each bucket's grid
    /// capital, rounded to the cent once. Can be negative.
    pub unrealized_pnl: Vec<Dollars>,
}

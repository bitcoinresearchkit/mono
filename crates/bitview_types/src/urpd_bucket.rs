use brk_types::{Bitcoin, Dollars};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single bucket in a URPD snapshot.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UrpdBucket {
    /// The bucket's price, in USD: its lower bound when aggregated, and for `Raw` the price its
    /// coins' creation prices round to (four significant digits of the price in cents).
    pub price_floor: Dollars,
    /// Supply held with a last-move price inside this bucket, in BTC. With a `cointime` or
    /// `coinflow` weight, the weighted supply, floored to whole satoshis at each grid price.
    pub supply: Bitcoin,
    /// Realized cap contribution in USD: `price * supply` summed over the coins in this bucket,
    /// each coin's creation price taken on the price grid (the `Raw` bucket price, within 0.05%
    /// of the exact price), rounded to the cent: a bucket worth under half a cent shows 0. The
    /// buckets therefore need not sum to the cohort's realized cap series.
    pub realized_cap: Dollars,
    /// Unrealized P&L in USD against the close on the snapshot date: `close * supply` minus the
    /// bucket's grid capital, rounded to the cent once. Can be negative.
    pub unrealized_pnl: Dollars,
}

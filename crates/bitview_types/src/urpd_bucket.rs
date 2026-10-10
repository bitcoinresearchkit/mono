use brk_types::{Bitcoin, Dollars};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single bucket in a URPD snapshot.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UrpdBucket {
    /// The bucket's price, in USD: its lower bound when aggregated, and for `Raw` the price its
    /// coins' creation prices round to (four significant digits of the price in cents).
    pub price_floor: Dollars,
    /// Supply held with a last-move price inside this bucket, in BTC.
    pub supply: Bitcoin,
    /// Realized cap contribution in USD: sum of `realized_price * supply` over the coins in this bucket.
    pub realized_cap: Dollars,
    /// Unrealized P&L in USD against the close on the snapshot date: `close * supply - realized_cap`. Can be negative.
    pub unrealized_pnl: Dollars,
}

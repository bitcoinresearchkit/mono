use brk_types::{Dollars, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Current price response matching mempool.space /api/v1/prices format
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Prices {
    /// Unix timestamp
    pub time: Timestamp,
    /// BTC/USD price
    #[serde(rename = "USD")]
    pub usd: Dollars,
}

/// Historical price response
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalPrice {
    /// Price data points
    pub prices: Vec<HistoricalPriceEntry>,
    /// Exchange rates (currently empty)
    #[serde(rename = "exchangeRates")]
    pub exchange_rates: ExchangeRates,
}

/// A single price data point
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalPriceEntry {
    /// Unix timestamp
    pub time: Timestamp,
    /// BTC/USD price
    #[serde(rename = "USD")]
    pub usd: Dollars,
}

/// Exchange rates (USD base, on-chain only — no fiat pairs available)
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ExchangeRates {}

use brk_types::{Dollars, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A BTC/USD price at a time: the current price (`/api/v1/prices`) or one historical close
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PricePoint {
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
    pub prices: Vec<PricePoint>,
    /// Exchange rates (currently empty)
    #[serde(rename = "exchangeRates")]
    pub exchange_rates: ExchangeRates,
}

/// Exchange rates (USD base, on-chain only — no fiat pairs available)
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ExchangeRates {}

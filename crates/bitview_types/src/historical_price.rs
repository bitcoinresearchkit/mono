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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalPrice {
    pub prices: Vec<PricePoint>,
    /// Exchange rates (currently empty)
    #[serde(rename = "exchangeRates")]
    pub exchange_rates: ExchangeRates,
}

/// Always empty: prices are quoted in USD only.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct ExchangeRates {}

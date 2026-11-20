use brk_types::{Dollars, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A BTC/USD price at a time: the current price (`/api/v1/prices`) or one historical close
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(transform = crate::emitted::every_key_required)]
pub struct PricePoint {
    /// Unix timestamp: a historical close's, or for the current price when it was derived (the
    /// mempool publication counted in it, else the tip block's time)
    pub time: Timestamp,
    /// BTC/USD price
    #[serde(rename = "USD")]
    pub usd: Option<Dollars>,
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
#[serde(deny_unknown_fields)]
pub struct ExchangeRates {}

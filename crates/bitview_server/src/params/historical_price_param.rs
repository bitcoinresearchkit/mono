use schemars::JsonSchema;
use serde::Deserialize;

use bitview_types::Currency;
use brk_types::Timestamp;

/// Historical price query parameters, as mempool.space's
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HistoricalPriceParam {
    /// UNIX timestamp: the latest close at or before it
    pub timestamp: Option<Timestamp>,
    /// Quote currency: USD, the only one served
    #[allow(dead_code, reason = "deserializing it is the check")]
    pub currency: Option<Currency>,
}

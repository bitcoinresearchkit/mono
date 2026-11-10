use schemars::JsonSchema;
use serde::Deserialize;

/// A bounded page of the mempool's full transaction IDs.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MempoolTxidsQuery {
    /// Transaction IDs to return, at most 10,000 (the default).
    #[serde(default, deserialize_with = "bitview_types::de_plain")]
    pub max_txs: Option<usize>,
}

use brk_types::{Address, Txid};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddressSummaryParam {
    #[serde(rename = "address")]
    pub addr: Address,
    /// Last transaction from the previous summary page (excluded from this page).
    pub last_seen_txid: Txid,
}

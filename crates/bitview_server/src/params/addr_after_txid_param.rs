use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::{Address, Txid};

#[derive(Deserialize, JsonSchema)]
pub struct AddressAfterTxidParam {
    #[serde(rename = "address")]
    pub addr: Address,

    /// Last txid from the previous page (return transactions strictly older than this)
    pub after_txid: Txid,
}

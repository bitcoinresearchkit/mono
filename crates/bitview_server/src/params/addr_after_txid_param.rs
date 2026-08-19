use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::{Addr, Txid};

/// Bitcoin address + last-seen txid path parameters (Esplora-style pagination)
#[derive(Deserialize, JsonSchema)]
pub struct AddrAfterTxidParam {
    #[serde(rename = "address")]
    pub addr: Addr,

    /// Last txid from the previous page (return transactions strictly older than this)
    pub after_txid: Txid,
}

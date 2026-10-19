use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::{Txid, Vout};

#[derive(Deserialize, JsonSchema)]
pub struct TxidVout {
    pub txid: Txid,
    /// Output index
    pub vout: Vout,
}

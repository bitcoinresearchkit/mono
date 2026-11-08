use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::{Txid, Vout};

#[derive(Deserialize, JsonSchema)]
pub struct TxidVout {
    pub txid: Txid,
    /// Output index
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub vout: Vout,
}

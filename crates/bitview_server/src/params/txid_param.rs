use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Txid;

#[derive(Deserialize, JsonSchema)]
pub struct TxidParam {
    pub txid: Txid,
}

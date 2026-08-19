use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Txid;

/// Transaction ID path parameter
#[derive(Deserialize, JsonSchema)]
pub struct TxidParam {
    pub txid: Txid,
}

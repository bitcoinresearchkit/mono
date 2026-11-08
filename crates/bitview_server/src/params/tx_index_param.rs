use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::TxIndex;

#[derive(Deserialize, JsonSchema)]
pub struct TxIndexParam {
    /// Global transaction index across the chain
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub index: TxIndex,
}

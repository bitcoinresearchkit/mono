use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::TxIndex;

#[derive(Deserialize, JsonSchema)]
pub struct TxIndexParam {
    /// Global transaction index across the chain
    pub index: TxIndex,
}

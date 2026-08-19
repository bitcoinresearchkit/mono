use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::TxIndex;

/// Transaction index path parameter
#[derive(Deserialize, JsonSchema)]
pub struct TxIndexParam {
    pub index: TxIndex,
}

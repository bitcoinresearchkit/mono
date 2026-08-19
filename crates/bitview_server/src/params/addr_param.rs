use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Addr;

/// Bitcoin address path parameter
#[derive(Deserialize, JsonSchema)]
pub struct AddrParam {
    #[serde(rename = "address")]
    pub addr: Addr,
}

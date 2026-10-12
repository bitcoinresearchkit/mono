use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::Address;

/// Bitcoin address path parameter
#[derive(Deserialize, JsonSchema)]
pub struct AddressParam {
    #[serde(rename = "address")]
    pub addr: Address,
}

use brk_types::{Address, AddressOutputType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddressHashPrefixMatches {
    pub address_type: AddressOutputType,
    /// The requested prefix, lowercased.
    #[schemars(extend("pattern" = "^[0-9a-f]{1,16}$"))]
    pub prefix: String,
    pub truncated: bool,
    pub addresses: Vec<Address>,
}

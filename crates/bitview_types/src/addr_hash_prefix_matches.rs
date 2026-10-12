use brk_types::{Address, OutputType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddressHashPrefixMatches {
    pub address_type: OutputType,
    pub prefix: String,
    pub truncated: bool,
    pub addresses: Vec<Address>,
}

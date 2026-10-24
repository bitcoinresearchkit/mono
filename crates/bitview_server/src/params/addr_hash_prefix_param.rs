use brk_types::AddressOutputType;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct AddressHashPrefixParam {
    pub address_type: AddressOutputType,
    /// First 1–16 hexadecimal nibbles of the RapidHash v3 hash over the raw
    /// address payload bytes.
    #[schemars(extend("pattern" = "^[0-9a-f]{1,16}$"))]
    pub prefix: String,
}

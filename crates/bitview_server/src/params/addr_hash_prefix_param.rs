use brk_types::OutputType;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct AddrHashPrefixParam {
    pub addr_type: OutputType,
    /// First 1–16 hexadecimal nibbles of the RapidHash v3 hash over the raw
    /// address payload bytes.
    pub prefix: String,
}

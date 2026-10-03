use brk_types::{Addr, OutputType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddrHashPrefixMatches {
    pub addr_type: OutputType,
    pub prefix: String,
    pub truncated: bool,
    pub addresses: Vec<Addr>,
}

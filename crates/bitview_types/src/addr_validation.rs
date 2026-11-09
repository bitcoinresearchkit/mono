use brk_types::{Address, Hex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressValidation {
    #[schemars(example = true)]
    pub isvalid: bool,

    /// The validated address
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "address")]
    pub addr: Option<Address>,

    /// The scriptPubKey in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: Option<Hex>,

    /// Whether the address is P2SH, P2WSH or P2TR
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isscript: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub iswitness: Option<bool>,

    /// Witness version (0 for P2WPKH/P2WSH, 1 for P2TR, 2 to 16 for future versions; none for P2A)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_version: Option<u8>,

    /// Witness program in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_program: Option<Hex>,

    /// Positions of the characters at fault in an invalid address (Bitcoin Core's), possibly none;
    /// omitted for valid ones
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_locations: Option<Vec<usize>>,

    /// Error message for invalid addresses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

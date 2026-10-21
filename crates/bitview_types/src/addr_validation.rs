use bitcoin::hex::DisplayHex;
use brk_types::{Address, AddressBytes, Hex, OutputType};
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

    /// Witness version (0 for P2WPKH/P2WSH, 1 for P2TR and P2A)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_version: Option<u8>,

    /// Witness program in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_program: Option<Hex>,

    /// Error locations: an empty array for invalid addresses, omitted for valid ones
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_locations: Option<Vec<usize>>,

    /// Error message for invalid addresses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl AddressValidation {
    fn invalid(error: String) -> Self {
        Self {
            isvalid: false,
            addr: None,
            script_pub_key: None,
            isscript: None,
            iswitness: None,
            witness_version: None,
            witness_program: None,
            error_locations: Some(vec![]),
            error: Some(error),
        }
    }

    pub fn from_addr(addr: &str) -> Self {
        let script = match AddressBytes::addr_to_script(addr) {
            Ok(s) => s,
            Err(e) => return Self::invalid(e.to_string()),
        };

        let output_type = OutputType::from(&script);
        let script_hex = Hex::from(script.as_bytes().to_lower_hex_string());

        let is_script = matches!(
            output_type,
            OutputType::P2SH | OutputType::P2WSH | OutputType::P2TR
        );
        let witness_version = script.witness_version().map(|v| v.to_num());
        let is_witness = witness_version.is_some();
        // A witness program is the version opcode, one push opcode, then the program.
        let witness_program =
            is_witness.then(|| Hex::from(script.as_bytes()[2..].to_lower_hex_string()));

        Self {
            isvalid: true,
            addr: Some(Address::from(addr.to_string())),
            script_pub_key: Some(script_hex),
            isscript: Some(is_script),
            iswitness: Some(is_witness),
            witness_version,
            witness_program,
            error_locations: None,
            error: None,
        }
    }
}

use bitcoin::hex::DisplayHex;
use brk_types::{AddrBytes, OutputType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Address validation result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddrValidation {
    /// Whether the address is valid
    #[schemars(example = true)]
    isvalid: bool,

    /// The validated address
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "address")]
    addr: Option<String>,

    /// The scriptPubKey in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "scriptPubKey")]
    script_pub_key: Option<String>,

    /// Whether this is a script address (P2SH)
    #[serde(skip_serializing_if = "Option::is_none")]
    isscript: Option<bool>,

    /// Whether this is a witness address
    #[serde(skip_serializing_if = "Option::is_none")]
    iswitness: Option<bool>,

    /// Witness version (0 for P2WPKH/P2WSH, 1 for P2TR)
    #[serde(skip_serializing_if = "Option::is_none")]
    witness_version: Option<u8>,

    /// Witness program in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    witness_program: Option<String>,

    /// Error locations (empty array for most errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    error_locations: Option<Vec<usize>>,

    /// Error message for invalid addresses
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl AddrValidation {
    /// Returns an invalid validation result with error detail
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

    /// Validate a Bitcoin address string and return details
    pub fn from_addr(addr: &str) -> Self {
        let script = match AddrBytes::addr_to_script(addr) {
            Ok(s) => s,
            Err(e) => return Self::invalid(e.to_string()),
        };

        let output_type = OutputType::from(&script);
        let script_hex = script.as_bytes().to_lower_hex_string();

        let is_script = matches!(
            output_type,
            OutputType::P2SH | OutputType::P2WSH | OutputType::P2TR
        );
        let is_witness = matches!(
            output_type,
            OutputType::P2WPKH | OutputType::P2WSH | OutputType::P2TR | OutputType::P2A
        );

        let (witness_version, witness_program) = if is_witness {
            let version = script.witness_version().map(|v| v.to_num());
            let program = if script.len() > 2 {
                Some(script.as_bytes()[2..].to_lower_hex_string())
            } else {
                None
            };
            (version, program)
        } else {
            (None, None)
        };

        Self {
            isvalid: true,
            addr: Some(addr.to_string()),
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

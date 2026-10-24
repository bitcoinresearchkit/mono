#[cfg(feature = "schemars")]
use std::borrow::Cow;
use std::fmt;

#[cfg(feature = "schemars")]
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};

use crate::{AddressBytes, OutputType};

/// The output type of an address: any `OutputType` but `p2ms`, `op_return`, `empty_script` and
/// `unknown_script`. Parsing another type fails, naming the address types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "OutputType", into = "OutputType")]
pub struct AddressOutputType(OutputType);

impl AddressOutputType {
    pub const ALL: [Self; 8] = [
        Self(OutputType::P2PK65),
        Self(OutputType::P2PK33),
        Self(OutputType::P2PKH),
        Self(OutputType::P2SH),
        Self(OutputType::P2WPKH),
        Self(OutputType::P2WSH),
        Self(OutputType::P2TR),
        Self(OutputType::P2A),
    ];

    pub const fn output_type(self) -> OutputType {
        self.0
    }

    /// The length of this type's address payload, the bytes the hash-prefix index hashes.
    pub fn payload_len(self) -> usize {
        self.0
            .payload_len()
            .expect("every address type has a payload")
    }
}

impl From<&AddressBytes> for AddressOutputType {
    fn from(bytes: &AddressBytes) -> Self {
        Self(OutputType::from(bytes))
    }
}

impl TryFrom<OutputType> for AddressOutputType {
    type Error = String;

    fn try_from(output_type: OutputType) -> Result<Self, Self::Error> {
        debug_assert_eq!(
            Self::ALL.iter().any(|t| t.0 == output_type),
            output_type.is_addr()
        );
        if output_type.is_addr() {
            Ok(Self(output_type))
        } else {
            let names: Vec<String> = Self::ALL.iter().map(ToString::to_string).collect();
            Err(format!(
                "{output_type} has no addresses, expected one of {}",
                names.join(", ")
            ))
        }
    }
}

impl From<AddressOutputType> for OutputType {
    fn from(value: AddressOutputType) -> Self {
        value.0
    }
}

impl fmt::Display for AddressOutputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(feature = "schemars")]
impl JsonSchema for AddressOutputType {
    fn schema_name() -> Cow<'static, str> {
        "AddressOutputType".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        let names: Vec<String> = Self::ALL.iter().map(ToString::to_string).collect();
        json_schema!({
            "description": "The output type of an address: any output type but `p2ms`, `op_return`, \
                `empty_script` and `unknown_script`.",
            "type": "string",
            "enum": names,
        })
    }
}

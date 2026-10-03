use std::{fmt, str::FromStr};

use bitcoin::{Address, Network, ScriptBuf};
use brk_error::Error;
use derive_more::Deref;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::OutputType;
use crate::AddrBytes;

#[cfg(feature = "storage")]
use vecdb::Formattable;

/// Bitcoin address string
#[derive(Debug, Clone, Deref, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(transparent)]
#[cfg_attr(feature = "schemars", schemars(
    example = &"04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
    example = &"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
    example = &"bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"
))]
pub struct Addr(String);

impl From<String> for Addr {
    #[inline]
    fn from(addr: String) -> Self {
        Self(addr)
    }
}

impl TryFrom<&ScriptBuf> for Addr {
    type Error = Error;
    fn try_from(script: &ScriptBuf) -> Result<Self, Self::Error> {
        Self::try_from((script, OutputType::from(script)))
    }
}

impl TryFrom<(&ScriptBuf, OutputType)> for Addr {
    type Error = Error;
    fn try_from((script, output_type): (&ScriptBuf, OutputType)) -> Result<Self, Self::Error> {
        match output_type {
            OutputType::P2PK65 | OutputType::P2PK33 => {
                // P2PK has no standard address encoding, use raw pubkey hex
                let bytes = AddrBytes::try_from((script, output_type))?;
                Ok(Self(bytes_to_hex(bytes.as_slice())))
            }
            _ if output_type.is_addr() => {
                let addr = Address::from_script(script, Network::Bitcoin)
                    .map_err(|_| Error::InvalidAddr)?;
                Ok(Self(addr.to_string()))
            }
            _ => Err(Error::InvalidAddr),
        }
    }
}

impl TryFrom<&AddrBytes> for Addr {
    type Error = Error;
    fn try_from(bytes: &AddrBytes) -> Result<Self, Self::Error> {
        Self::try_from(&bytes.to_script_pubkey())
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex_string = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut hex_string, "{:02x}", byte).unwrap();
    }
    hex_string
}

impl FromStr for Addr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = AddrBytes::addr_to_script(s)?;
        Ok(Self(s.to_string()))
    }
}

#[cfg(feature = "storage")]
impl Formattable for Addr {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.0.as_bytes());
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

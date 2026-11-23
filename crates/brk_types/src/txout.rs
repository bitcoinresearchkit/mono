#[cfg(feature = "schemars")]
use std::borrow::Cow;

use bitcoin::{ScriptBuf, TxOut as BitcoinTxOut};
#[cfg(feature = "schemars")]
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::Deserialize;
#[cfg(feature = "schemars")]
use serde_json::json;

#[cfg(feature = "schemars")]
use crate::{Address, Hex, OutputTypeNormalized};
use crate::{AddressBytes, OutputType, Sats};

#[derive(Debug, Clone, Deserialize)]
pub struct TxOut {
    #[serde(rename = "scriptpubkey")]
    pub script_pubkey: ScriptBuf,
    pub value: Sats,
}

#[cfg(feature = "schemars")]
#[allow(dead_code)]
#[derive(JsonSchema)]
struct TxOutSchema {
    /// Script pubkey (locking script), encoded as hexadecimal.
    #[schemars(example = "00143b064c595a95f977f00352d6e917501267cacdc6")]
    scriptpubkey: Hex,
    /// Script pubkey in assembly format.
    #[schemars(example = "OP_0 OP_PUSHBYTES_20 3b064c595a95f977f00352d6e917501267cacdc6")]
    scriptpubkey_asm: String,
    /// Esplora/mempool.space script type.
    #[schemars(example = &"v0_p2wpkh")]
    scriptpubkey_type: OutputTypeNormalized,
    /// Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).
    #[schemars(example = &"bc1q8vryck26jhuh0uqr2ttwj96szfnu4nwxfmu39y")]
    scriptpubkey_address: Address,
    /// Value of the output in satoshis.
    #[schemars(example = Sats::new(7782))]
    value: Sats,
}

#[cfg(feature = "schemars")]
impl JsonSchema for TxOut {
    fn schema_name() -> Cow<'static, str> {
        "TxOut".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let mut schema = TxOutSchema::json_schema(generator);
        schema.insert(
            "required".to_owned(),
            json!([
                "scriptpubkey",
                "scriptpubkey_asm",
                "scriptpubkey_type",
                "value"
            ]),
        );
        schema
    }
}

impl TxOut {
    pub fn addr_bytes(&self) -> Option<AddressBytes> {
        AddressBytes::try_from(&self.script_pubkey).ok()
    }

    pub fn output_type(&self) -> OutputType {
        OutputType::from(&self.script_pubkey)
    }
}

impl From<BitcoinTxOut> for TxOut {
    #[inline]
    fn from(txout: BitcoinTxOut) -> Self {
        Self {
            script_pubkey: txout.script_pubkey,
            value: txout.value.into(),
        }
    }
}

impl From<&TxOut> for BitcoinTxOut {
    #[inline]
    fn from(txout: &TxOut) -> Self {
        Self {
            value: txout.value.into(),
            script_pubkey: txout.script_pubkey.clone(),
        }
    }
}

impl From<(ScriptBuf, Sats)> for TxOut {
    #[inline]
    fn from((script, value): (ScriptBuf, Sats)) -> Self {
        Self {
            script_pubkey: script,
            value,
        }
    }
}

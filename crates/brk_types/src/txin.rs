#[cfg(feature = "schemars")]
use std::borrow::Cow;

use bitcoin::{ScriptBuf, Sequence, TxIn as BitcoinTxIn, Witness, transaction::OutPoint};
#[cfg(feature = "schemars")]
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::Deserialize;
#[cfg(feature = "schemars")]
use serde_json::json;

#[cfg(feature = "schemars")]
use crate::Hex;
use crate::{TxOut, Txid, Vout};

#[derive(Debug, Clone, Deserialize)]
pub struct TxIn {
    pub txid: Txid,
    pub vout: Vout,
    pub prevout: Option<TxOut>,
    #[serde(rename = "scriptsig")]
    pub script_sig: ScriptBuf,
    #[serde(default)]
    pub witness: Witness,
    pub is_coinbase: bool,
    pub sequence: u32,
}

#[cfg(feature = "schemars")]
#[allow(dead_code)]
#[derive(JsonSchema)]
struct TxInSchema {
    /// Transaction ID of the output being spent
    #[schemars(example = "0000000000000000000000000000000000000000000000000000000000000000")]
    txid: Txid,
    /// Output index being spent (coinbase: 4294967295)
    #[schemars(example = 0)]
    vout: Vout,
    /// The output being spent
    prevout: Option<TxOut>,
    /// Signature script in hex (empty for native SegWit inputs)
    scriptsig: Hex,
    /// Signature script in assembly format
    scriptsig_asm: String,
    /// Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
    witness: Vec<Hex>,
    #[schemars(example = false)]
    is_coinbase: bool,
    #[schemars(example = 4294967293_u32)]
    sequence: u32,
    /// Inner redeemscript in assembly (P2SH inputs)
    inner_redeemscript_asm: String,
    /// Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)
    inner_witnessscript_asm: String,
}

#[cfg(feature = "schemars")]
impl JsonSchema for TxIn {
    fn schema_name() -> Cow<'static, str> {
        "TxIn".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let mut schema = TxInSchema::json_schema(generator);
        schema.insert(
            "required".to_owned(),
            json!([
                "txid",
                "vout",
                "prevout",
                "scriptsig",
                "scriptsig_asm",
                "is_coinbase",
                "sequence"
            ]),
        );
        schema
    }
}

impl From<&TxIn> for BitcoinTxIn {
    #[inline]
    fn from(txin: &TxIn) -> Self {
        Self {
            previous_output: OutPoint {
                txid: (&txin.txid).into(),
                vout: u32::from(txin.vout),
            },
            script_sig: txin.script_sig.clone(),
            sequence: Sequence(txin.sequence),
            witness: txin.witness.clone(),
        }
    }
}

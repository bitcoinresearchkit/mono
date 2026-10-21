#[cfg(feature = "schemars")]
use std::borrow::Cow;

use bitcoin::{Script, ScriptBuf, Sequence, TxIn as BitcoinTxIn, Witness, transaction::OutPoint};
#[cfg(feature = "schemars")]
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};
#[cfg(feature = "schemars")]
use serde_json::json;

#[cfg(feature = "schemars")]
use crate::Hex;
use crate::{TxOut, Txid, Vout, bytes::DisplayStr, tx::p2sh_redeem_script};

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

impl Serialize for TxIn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let has_witness = !self.witness.is_empty();
        let prevout_script = self.prevout.as_ref().map(|p| p.script_pubkey.as_script());

        let is_p2sh = !self.is_coinbase && prevout_script.is_some_and(Script::is_p2sh);
        let redeem_script = is_p2sh
            .then(|| p2sh_redeem_script(&self.script_sig))
            .flatten();

        let inner_witness = prevout_script.and_then(|spk| {
            if spk.is_p2wsh() || redeem_script.is_some_and(Script::is_p2wsh) {
                self.witness.witness_script()
            } else if spk.is_p2tr() {
                self.witness.taproot_leaf_script().map(|leaf| leaf.script)
            } else {
                None
            }
        });

        let field_count =
            7 + has_witness as usize + is_p2sh as usize + inner_witness.is_some() as usize;

        let mut state = serializer.serialize_struct("TxIn", field_count)?;

        state.serialize_field("txid", &self.txid)?;
        state.serialize_field("vout", &self.vout)?;
        state.serialize_field("prevout", &self.prevout)?;
        state.serialize_field(
            "scriptsig",
            &DisplayStr(format_args!("{:x}", self.script_sig)),
        )?;
        state.serialize_field("scriptsig_asm", &DisplayStr(self.script_sig.as_script()))?;
        if has_witness {
            state.serialize_field("witness", &self.witness)?;
        }
        state.serialize_field("is_coinbase", &self.is_coinbase)?;
        state.serialize_field("sequence", &self.sequence)?;
        if is_p2sh {
            state.serialize_field(
                "inner_redeemscript_asm",
                &DisplayStr(redeem_script.unwrap_or(Script::new())),
            )?;
        }
        if let Some(script) = inner_witness {
            state.serialize_field("inner_witnessscript_asm", &DisplayStr(script))?;
        }

        state.end()
    }
}

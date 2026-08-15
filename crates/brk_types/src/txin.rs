use crate::{TxOut, Txid, Vout, Witness};
use bitcoin::{Script, ScriptBuf, Sequence, transaction::OutPoint};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

/// Transaction input
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[schemars(transform = txin_wire_schema)]
pub struct TxIn {
    /// Transaction ID of the output being spent
    #[schemars(example = "0000000000000000000000000000000000000000000000000000000000000000")]
    pub txid: Txid,

    /// Output index being spent (u16: coinbase is 65535, mempool.space uses u32: 4294967295)
    #[schemars(example = 0)]
    pub vout: Vout,

    /// Information about the previous output being spent
    pub prevout: Option<TxOut>,

    /// Signature script (hex, for non-SegWit inputs)
    #[schemars(rename = "scriptsig", with = "String")]
    pub script_sig: ScriptBuf,

    /// Signature script in assembly format
    #[schemars(rename = "scriptsig_asm", with = "String")]
    pub script_sig_asm: (),

    /// Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
    pub witness: Witness,

    /// Whether this input is a coinbase (block reward) input
    #[schemars(example = false)]
    pub is_coinbase: bool,

    /// Input sequence number
    #[schemars(example = 4294967293_u32)]
    pub sequence: u32,

    /// Inner redeemscript in assembly (for P2SH-wrapped SegWit: scriptsig + witness both present)
    #[schemars(rename = "inner_redeemscript_asm", with = "String")]
    pub inner_redeem_script_asm: (),

    /// Inner witnessscript in assembly (for P2WSH: last witness item decoded as script)
    #[schemars(rename = "inner_witnessscript_asm", with = "String")]
    pub inner_witness_script_asm: (),
}

fn txin_wire_schema(schema: &mut schemars::Schema) {
    let Some(required) = schema
        .get_mut("required")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return;
    };

    required.retain(|field| {
        !matches!(
            field.as_str(),
            Some("witness" | "inner_redeemscript_asm" | "inner_witnessscript_asm")
        )
    });

    if !required.iter().any(|field| field == "prevout") {
        let index = required
            .iter()
            .position(|field| field == "vout")
            .map_or(required.len(), |index| index + 1);
        required.insert(index, serde_json::json!("prevout"));
    }
}

/// Reconstruct a canonical `bitcoin::TxIn` from the stored brk shape.
/// Mempool txs are never coinbase, so `vout` (u16 in brk) always fits
/// the bitcoin protocol's u32 vout field via widening.
impl From<&TxIn> for bitcoin::TxIn {
    #[inline]
    fn from(txin: &TxIn) -> Self {
        Self {
            previous_output: OutPoint {
                txid: (&txin.txid).into(),
                vout: u32::from(txin.vout),
            },
            script_sig: txin.script_sig.clone(),
            sequence: Sequence(txin.sequence),
            witness: (&txin.witness).into(),
        }
    }
}

impl Serialize for TxIn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let has_witness = !self.witness.is_empty();
        let has_scriptsig = !self.script_sig.is_empty();

        // P2SH / P2SH-wrapped SegWit: extract redeemscript from scriptsig
        let is_p2sh = self
            .prevout
            .as_ref()
            .is_some_and(|p| p.script_pubkey.is_p2sh());
        let inner_redeem = if has_scriptsig && is_p2sh && !self.is_coinbase {
            self.script_sig
                .redeem_script()
                .map(|s| s.to_asm_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        // P2WSH/P2SH-P2WSH: last witness item is the witnessScript
        // P2TR script path: second-to-last is the script, last is the control block
        let is_p2tr = self
            .prevout
            .as_ref()
            .is_some_and(|p| p.script_pubkey.is_p2tr());
        let inner_witness = if has_witness && self.witness.len() > 2 {
            let script_bytes = if is_p2tr {
                self.witness.second_to_last()
            } else {
                self.witness.last()
            };
            script_bytes
                .map(|b| Script::from_bytes(b).to_asm_string())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let has_inner_redeem = is_p2sh && !self.is_coinbase;
        let has_inner_witness = !inner_witness.is_empty();
        let field_count =
            7 + has_witness as usize + has_inner_redeem as usize + has_inner_witness as usize;

        let mut state = serializer.serialize_struct("TxIn", field_count)?;

        state.serialize_field("txid", &self.txid)?;
        state.serialize_field("vout", &self.vout)?;
        state.serialize_field("prevout", &self.prevout)?;
        state.serialize_field("scriptsig", &self.script_sig.to_hex_string())?;
        state.serialize_field("scriptsig_asm", &self.script_sig.to_asm_string())?;
        if has_witness {
            state.serialize_field("witness", &self.witness)?;
        }
        state.serialize_field("is_coinbase", &self.is_coinbase)?;
        state.serialize_field("sequence", &self.sequence)?;
        if has_inner_redeem {
            state.serialize_field("inner_redeemscript_asm", &inner_redeem)?;
        }
        if has_inner_witness {
            state.serialize_field("inner_witnessscript_asm", &inner_witness)?;
        }

        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_marks_only_unconditional_wire_fields_as_required() {
        let schema = serde_json::to_value(schemars::schema_for!(TxIn)).unwrap();
        let required = schema["required"].as_array().unwrap();
        for field in [
            "txid",
            "vout",
            "prevout",
            "scriptsig",
            "scriptsig_asm",
            "is_coinbase",
            "sequence",
        ] {
            assert!(
                required.iter().any(|value| value == field),
                "{field} should be required; schema: {schema}"
            );
        }
        for field in [
            "witness",
            "inner_redeemscript_asm",
            "inner_witnessscript_asm",
        ] {
            assert!(!required.iter().any(|value| value == field));
            assert!(schema["properties"].get(field).is_some());
        }
    }
}

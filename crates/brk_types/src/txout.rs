use crate::{Addr, AddrBytes, OutputType, Sats};
use bitcoin::ScriptBuf;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

/// Transaction output
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TxOut {
    /// Script pubkey (locking script)
    #[serde(
        rename = "scriptpubkey",
        serialize_with = "serialize_with_script_pubkey"
    )]
    #[schemars(
        with = "String",
        example = "00143b064c595a95f977f00352d6e917501267cacdc6"
    )]
    pub script_pubkey: ScriptBuf,

    /// Script pubkey in assembly format
    #[allow(dead_code)]
    #[serde(skip, rename = "scriptpubkey_asm")]
    #[schemars(
        with = "String",
        example = "OP_0 OP_PUSHBYTES_20 3b064c595a95f977f00352d6e917501267cacdc6"
    )]
    script_pubkey_asm: (),

    /// Esplora/mempool.space script type
    #[allow(dead_code)]
    #[serde(skip, rename = "scriptpubkey_type")]
    #[schemars(with = "crate::OutputTypeNormalized", example = &"v0_p2wpkh")]
    script_pubkey_type: (),

    /// Bitcoin address (if applicable, None for OP_RETURN)
    #[allow(dead_code)]
    #[serde(skip, rename = "scriptpubkey_address")]
    #[schemars(with = "Option<Addr>", example = Some("bc1q8vryck26jhuh0uqr2ttwj96szfnu4nwxfmu39y".to_string()))]
    script_pubkey_addr: (),

    /// Value of the output in satoshis
    #[schemars(example = Sats::new(7782))]
    pub value: Sats,
}

impl TxOut {
    pub fn addr(&self) -> Option<Addr> {
        Addr::try_from(&self.script_pubkey).ok()
    }

    pub fn addr_bytes(&self) -> Option<AddrBytes> {
        AddrBytes::try_from(&self.script_pubkey).ok()
    }

    pub fn type_(&self) -> OutputType {
        OutputType::from(&self.script_pubkey)
    }

    pub fn script_pubkey_asm(&self) -> String {
        self.script_pubkey.to_asm_string()
    }
}

impl From<bitcoin::TxOut> for TxOut {
    #[inline]
    fn from(txout: bitcoin::TxOut) -> Self {
        Self {
            script_pubkey: txout.script_pubkey,
            value: txout.value.into(),
            script_pubkey_asm: (),
            script_pubkey_addr: (),
            script_pubkey_type: (),
        }
    }
}

impl From<&TxOut> for bitcoin::TxOut {
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
            script_pubkey_addr: (),
            script_pubkey_asm: (),
            script_pubkey_type: (),
            value,
        }
    }
}

impl Serialize for TxOut {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let output_type = self.type_();
        let addr = self.addr();
        let field_count = if addr.is_some() { 5 } else { 4 };
        let mut state = serializer.serialize_struct("TxOut", field_count)?;
        state.serialize_field("scriptpubkey", &self.script_pubkey.to_hex_string())?;
        state.serialize_field("scriptpubkey_asm", &self.script_pubkey_asm())?;
        state.serialize_field("scriptpubkey_type", &output_type.normalized())?;
        if let Some(addr) = &addr {
            state.serialize_field("scriptpubkey_address", addr)?;
        }
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p2pk_script(pubkey: &[u8]) -> ScriptBuf {
        let mut script = Vec::with_capacity(pubkey.len() + 2);
        script.push(pubkey.len() as u8);
        script.extend_from_slice(pubkey);
        script.push(0xac);
        ScriptBuf::from_bytes(script)
    }

    fn script_type(script: ScriptBuf) -> (OutputType, String) {
        let txout = TxOut::from((script, Sats::new(0)));
        let output_type = txout.type_();
        let value = serde_json::to_value(txout).unwrap();
        let script_type = value["scriptpubkey_type"].as_str().unwrap().to_owned();
        (output_type, script_type)
    }

    #[test]
    fn script_type_uses_normalized_names() {
        let p2pk33 = [
            0x02, 0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce,
            0x87, 0x0b, 0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81,
            0x5b, 0x16, 0xf8, 0x17, 0x98,
        ];
        let p2pk65 = [
            0x04, 0x67, 0x8a, 0xfd, 0xb0, 0xfe, 0x55, 0x48, 0x27, 0x19, 0x67, 0xf1, 0xa6, 0x71,
            0x30, 0xb7, 0x10, 0x5c, 0xd6, 0xa8, 0x28, 0xe0, 0x39, 0x09, 0xa6, 0x79, 0x62, 0xe0,
            0xea, 0x1f, 0x61, 0xde, 0xb6, 0x49, 0xf6, 0xbc, 0x3f, 0x4c, 0xef, 0x38, 0xc4, 0xf3,
            0x55, 0x04, 0xe5, 0x1e, 0xc1, 0x12, 0xde, 0x5c, 0x38, 0x4d, 0xf7, 0xba, 0x0b, 0x8d,
            0x57, 0x8a, 0x4c, 0x70, 0x2b, 0x6b, 0xf1, 0x1d, 0x5f,
        ];

        assert_eq!(
            script_type(p2pk_script(&p2pk33)),
            (OutputType::P2PK33, "p2pk".to_string())
        );
        assert_eq!(
            script_type(p2pk_script(&p2pk65)),
            (OutputType::P2PK65, "p2pk".to_string())
        );
        let mut p2wpkh = vec![0; 22];
        p2wpkh[1] = 0x14;
        assert_eq!(
            script_type(ScriptBuf::from_bytes(p2wpkh)),
            (OutputType::P2WPKH, "v0_p2wpkh".to_string())
        );
        assert_eq!(
            script_type(ScriptBuf::from_bytes(vec![0x6a])),
            (OutputType::OpReturn, "op_return".to_string())
        );
    }
}

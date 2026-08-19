use bitcoin::{
    Script,
    opcodes::all::{OP_CHECKMULTISIG, OP_CHECKMULTISIGVERIFY, OP_CHECKSIG, OP_CHECKSIGVERIFY},
    script::Instruction,
};
use brk_types::{OpReturnKind, StoredU32};

#[derive(Debug, Clone, Copy)]
pub struct Facts {
    pub kind: OpReturnKind,
    pub legacy_sigops: StoredU32,
    pub post_op_return_bytes: StoredU32,
}

pub fn analyze(script: &Script) -> Facts {
    let data = &script.as_bytes()[1..];
    let (prefix, legacy_sigops) = scan(script);
    Facts {
        kind: classify(data, prefix),
        legacy_sigops: StoredU32::from(legacy_sigops),
        post_op_return_bytes: StoredU32::from(data.len()),
    }
}

pub fn classify(data: &[u8], prefix: Option<&[u8]>) -> OpReturnKind {
    if data.first() == Some(&0x5d) {
        return OpReturnKind::Runes;
    }

    let Some(prefix) = prefix else {
        return OpReturnKind::Empty;
    };

    if prefix.starts_with(b"omni") {
        OpReturnKind::Omni
    } else if prefix.starts_with(b"X2") || prefix.starts_with(b"X1") {
        OpReturnKind::Stacks
    } else if prefix.starts_with(b"id") {
        OpReturnKind::Blockstack
    } else if prefix.starts_with(b"CC") {
        OpReturnKind::Colu
    } else if prefix.starts_with(b"OA\x01\x00") {
        OpReturnKind::OpenAssets
    } else if prefix.starts_with(b"SPK") {
        OpReturnKind::CoinSpark
    } else if prefix.starts_with(b"POET") {
        OpReturnKind::Poet
    } else if prefix.starts_with(b"DOCPROOF") {
        OpReturnKind::Docproof
    } else if prefix.starts_with(b"\x05\x88\x96\x0d\x73\xd7\x19\x01") {
        OpReturnKind::OpenTimestamps
    } else if prefix.starts_with(b"Factom!!") {
        OpReturnKind::Factom
    } else if prefix.starts_with(b"EW") {
        OpReturnKind::EternityWall
    } else if is_memo(prefix) {
        OpReturnKind::Memo
    } else if prefix.starts_with(b"BP") {
        OpReturnKind::Bitproof
    } else if prefix.starts_with(b"ASCRIBE\0") {
        OpReturnKind::Ascribe
    } else if prefix.starts_with(b"Stampery") {
        OpReturnKind::Stampery
    } else if prefix.starts_with(b"EPOBC") {
        OpReturnKind::Epobc
    } else if data.len() == 82 {
        OpReturnKind::VeriBlock
    } else if (36..=38).contains(&data.len()) {
        OpReturnKind::Komodo
    } else if matches!(prefix.len(), 20 | 32) {
        OpReturnKind::BareHash
    } else if is_text(prefix) {
        OpReturnKind::Text
    } else {
        OpReturnKind::Unknown
    }
}

pub fn scan(script: &Script) -> (Option<&[u8]>, usize) {
    let mut first_push = None;
    let mut legacy_sigops = 0;

    for instruction in script.instructions().skip(1) {
        match instruction {
            Ok(Instruction::PushBytes(bytes)) => {
                if first_push.is_none() && !bytes.is_empty() {
                    first_push = Some(bytes.as_bytes());
                }
            }
            Ok(Instruction::Op(opcode)) => match opcode {
                OP_CHECKSIG | OP_CHECKSIGVERIFY => legacy_sigops += 1,
                OP_CHECKMULTISIG | OP_CHECKMULTISIGVERIFY => legacy_sigops += 20,
                _ => {}
            },
            Err(_) => break,
        }
    }

    (first_push, legacy_sigops)
}

pub fn is_memo(prefix: &[u8]) -> bool {
    prefix.len() >= 2
        && prefix[0] == 0x6d
        && (matches!(prefix[1], 0x01..=0x07) || prefix[1] == 0x0c)
}

pub fn is_text(prefix: &[u8]) -> bool {
    prefix.len() >= 4
        && prefix
            .iter()
            .filter(|byte| byte.is_ascii_graphic() || **byte == b' ')
            .count()
            * 10
            >= prefix.len() * 9
}

#[cfg(test)]
mod tests {
    use bitcoin::ScriptBuf;
    use bitcoin::opcodes::all::OP_RETURN;
    use bitcoin::script::{Builder, PushBytesBuf};

    use super::*;

    pub fn pushed(data: &[u8]) -> ScriptBuf {
        Builder::new()
            .push_opcode(OP_RETURN)
            .push_slice(PushBytesBuf::try_from(data.to_vec()).unwrap())
            .into_script()
    }

    #[test]
    pub fn classifies_exact_prefix_before_heuristics() {
        let mut payload = b"omni".to_vec();
        payload.resize(80, 0);

        assert_eq!(analyze(&pushed(&payload)).kind, OpReturnKind::Omni);
    }

    #[test]
    pub fn classifies_runes_opcode() {
        assert_eq!(
            analyze(&ScriptBuf::from_bytes(vec![OP_RETURN.to_u8(), 0x5d])).kind,
            OpReturnKind::Runes
        );
    }

    #[test]
    pub fn classifies_empty_and_unknown() {
        assert_eq!(
            analyze(&ScriptBuf::from_bytes(vec![OP_RETURN.to_u8()])).kind,
            OpReturnKind::Empty
        );
        assert_eq!(
            analyze(&pushed(&[0, 1, 2, 3, 4])).kind,
            OpReturnKind::Unknown
        );
    }

    #[test]
    pub fn classifies_known_protocol_prefixes() {
        let cases: &[(&[u8], OpReturnKind)] = &[
            (b"omni", OpReturnKind::Omni),
            (b"X2", OpReturnKind::Stacks),
            (b"id", OpReturnKind::Blockstack),
            (b"CC", OpReturnKind::Colu),
            (b"OA\x01\x00", OpReturnKind::OpenAssets),
            (b"SPK", OpReturnKind::CoinSpark),
            (b"POET", OpReturnKind::Poet),
            (b"DOCPROOF", OpReturnKind::Docproof),
            (
                b"\x05\x88\x96\x0d\x73\xd7\x19\x01",
                OpReturnKind::OpenTimestamps,
            ),
            (b"Factom!!", OpReturnKind::Factom),
            (b"EW", OpReturnKind::EternityWall),
            (b"\x6d\x01", OpReturnKind::Memo),
            (b"BP", OpReturnKind::Bitproof),
            (b"ASCRIBE\0", OpReturnKind::Ascribe),
            (b"Stampery", OpReturnKind::Stampery),
            (b"EPOBC", OpReturnKind::Epobc),
        ];

        for (prefix, expected) in cases {
            assert_eq!(analyze(&pushed(prefix)).kind, *expected);
        }
    }

    #[test]
    pub fn classifies_length_and_content_heuristics() {
        assert_eq!(analyze(&pushed(&[1; 80])).kind, OpReturnKind::VeriBlock);
        assert_eq!(analyze(&pushed(&[1; 35])).kind, OpReturnKind::Komodo);
        assert_eq!(analyze(&pushed(&[1; 20])).kind, OpReturnKind::BareHash);
        assert_eq!(analyze(&pushed(b"plain text")).kind, OpReturnKind::Text);
    }

    #[test]
    pub fn counts_only_executed_legacy_sigop_opcodes() {
        use bitcoin::opcodes::all::{
            OP_CHECKMULTISIG, OP_CHECKMULTISIGVERIFY, OP_CHECKSIG, OP_CHECKSIGVERIFY,
        };

        let opcodes = ScriptBuf::from_bytes(vec![
            OP_RETURN.to_u8(),
            OP_CHECKSIG.to_u8(),
            OP_CHECKSIGVERIFY.to_u8(),
            OP_CHECKMULTISIG.to_u8(),
            OP_CHECKMULTISIGVERIFY.to_u8(),
        ]);
        assert_eq!(u32::from(analyze(&opcodes).legacy_sigops), 42);

        let pushed = pushed(&[
            OP_CHECKSIG.to_u8(),
            OP_CHECKSIGVERIFY.to_u8(),
            OP_CHECKMULTISIG.to_u8(),
            OP_CHECKMULTISIGVERIFY.to_u8(),
        ]);
        assert_eq!(u32::from(analyze(&pushed).legacy_sigops), 0);
    }
}

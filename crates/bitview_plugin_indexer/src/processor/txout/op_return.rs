use bitcoin::{
    Script,
    opcodes::all::{
        OP_CHECKMULTISIG, OP_CHECKMULTISIGVERIFY, OP_CHECKSIG, OP_CHECKSIGVERIFY, OP_PUSHNUM_13,
    },
    script::Instruction,
};
use bitview_primitives::{OpReturnKind, StoredU32};

#[derive(Debug, Clone, Copy)]
pub struct Facts {
    pub kind: OpReturnKind,
    pub legacy_sigops: StoredU32,
    pub post_op_return_bytes: StoredU32,
}

pub fn analyze(script: &Script) -> Facts {
    let data = &script.as_bytes()[1..];
    let (kind, legacy_sigops) = if data.first().copied() == Some(OP_PUSHNUM_13.to_u8()) {
        (OpReturnKind::Runes, script[1..].count_sigops_legacy())
    } else {
        let (prefix, legacy_sigops) = scan(script);
        (classify(data, prefix), legacy_sigops)
    };
    Facts {
        kind,
        legacy_sigops: StoredU32::from(legacy_sigops),
        post_op_return_bytes: StoredU32::from(data.len()),
    }
}

fn classify(data: &[u8], prefix: Option<&[u8]>) -> OpReturnKind {
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

fn scan(script: &Script) -> (Option<&[u8]>, usize) {
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

fn is_memo(prefix: &[u8]) -> bool {
    prefix.len() >= 2
        && prefix[0] == 0x6d
        && (matches!(prefix[1], 0x01..=0x07) || prefix[1] == 0x0c)
}

fn is_text(prefix: &[u8]) -> bool {
    prefix.len() >= 4
        && prefix
            .iter()
            .filter(|byte| byte.is_ascii_graphic() || **byte == b' ')
            .count()
            * 10
            >= prefix.len() * 9
}
